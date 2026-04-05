#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path


def fail(msg: str) -> int:
    print(f"[group-id-synthesis-report-selftest] fail: {msg}")
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
    with tempfile.TemporaryDirectory(prefix="group-id-synthesis-report-") as tmp:
        tmp_dir = Path(tmp)

        seed_report = tmp_dir / "seed_runtime_visual.detjson"
        seed_doc = run_json(
            root,
            ["node", "tests/seamgrim_seed_runtime_visual_pack_runner.mjs", "--json-out", str(seed_report)],
            seed_report,
        )
        if seed_doc.get("schema") != "ddn.seamgrim.seed_runtime_visual_pack_report.v1":
            return fail(f"seed schema mismatch: {seed_doc.get('schema')}")
        cases = seed_doc.get("cases") or []
        pendulum = next((row for row in cases if row.get("id") == "physics_pendulum_seed_v1"), None)
        if not isinstance(pendulum, dict):
            return fail("seed pendulum case missing")
        if pendulum.get("group_ids") != ["pendulum.rod", "pendulum.bob", "pendulum.pivot"]:
            return fail(f"seed pendulum group_ids mismatch: {pendulum.get('group_ids')}")

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
            return fail(f"scene schema mismatch: {scene_doc.get('schema')}")
        packs = scene_doc.get("packs") or []
        lesson = next((row for row in packs if str(row.get("pack_dir", "")).replace("\\", "/").endswith("pack/edu_pilot_phys_econ/lesson_phys_01")), None)
        if not isinstance(lesson, dict):
            return fail("scene/session lesson summary missing")
        if lesson.get("group_ids") != ["pendulum.baseline", "pendulum.variant"]:
            return fail(f"scene/session group_ids mismatch: {lesson.get('group_ids')}")

    print("[group-id-synthesis-report-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
