#!/usr/bin/env python3
"""Gate for Wave3B G6 student assignment open/run/save workflow."""

from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_wave3b_student_assignment_v1"


def run(cmd: list[str], timeout: int | None = None) -> str:
    proc = subprocess.run(
        cmd,
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        capture_output=True,
        timeout=timeout,
    )
    if proc.returncode != 0:
        detail = ((proc.stdout or "") + (proc.stderr or "")).strip()
        raise SystemExit(f"E_WAVE3B_G6_COMMAND: {' '.join(cmd)}\n{detail}")
    return ((proc.stdout or "") + (proc.stderr or "")).strip()


def main() -> None:
    contract = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    required = {
        "product_behavior_claim": True,
        "foundation_lesson_count": 12,
        "student_assignment_schema": "seamgrim.student_assignment_version.v0",
        "bundle_import_required": True,
        "prediction_run_control_explanation_required": True,
        "ddn_source_access_required": True,
        "student_version_save_reopen_required": True,
        "profile_allowed_controls_enforced": True,
        "no_runtime_role_toggle": True,
        "docs_ssot_change": False,
        "parser_change": False,
        "state_hash_formula_change": False,
    }
    for key, expected in required.items():
        actual = contract.get(key)
        if actual != expected:
            raise SystemExit(f"E_WAVE3B_G6_CONTRACT_{key}: expected {expected!r}, got {actual!r}")

    scripts = [
        "solutions/seamgrim_ui_mvp/ui/components/slider_panel.js",
        "solutions/seamgrim_ui_mvp/ui/screens/run.js",
        "tests/seamgrim_wave3b_student_assignment_runner.mjs",
    ]
    for script in scripts:
        run(["node", "--check", script], timeout=30)

    run(["python", "tests/run_pack_golden.py", "seamgrim_wave3b_student_assignment_v1"], timeout=120)
    output = run(["node", "tests/seamgrim_wave3b_student_assignment_runner.mjs"], timeout=600)
    if "seamgrim_wave3b_student_assignment: ok" not in output or '"checked":12' not in output:
        raise SystemExit(f"E_WAVE3B_G6_RUNNER_OUTPUT: {output}")
    print("[seamgrim-wave3b-student-assignment] OK behavior-closed")


if __name__ == "__main__":
    main()
