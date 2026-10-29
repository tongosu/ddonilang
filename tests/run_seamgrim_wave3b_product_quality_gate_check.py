#!/usr/bin/env python3
"""Gate for Wave3B G9 Foundation12 product/device/accessibility quality."""

from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_wave3b_product_quality_gate_v1"


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
        raise SystemExit(f"E_WAVE3B_G9_COMMAND: {' '.join(cmd)}\n{detail}")
    return ((proc.stdout or "") + (proc.stderr or "")).strip()


def main() -> None:
    contract = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    required = {
        "product_behavior_claim": True,
        "foundation_lesson_count": 12,
        "viewport_count": 4,
        "primary_view_nonblank_required": True,
        "semantic_binding_required": True,
        "teacher_student_teacher_workflow_required": True,
        "keyboard_run_required": True,
        "ddn_source_access_required": True,
        "a11y_names_required": True,
        "no_horizontal_overflow_required": True,
        "console_error_regression_required": True,
        "no_runtime_role_toggle": True,
        "docs_ssot_change": False,
        "parser_change": False,
        "state_hash_formula_change": False,
        "auto_score_claim": False,
        "education_effect_claim": False,
    }
    for key, expected in required.items():
        actual = contract.get(key)
        if actual != expected:
            raise SystemExit(f"E_WAVE3B_G9_CONTRACT_{key}: expected {expected!r}, got {actual!r}")
    if contract.get("required_viewports") != ["1440x900", "1280x800", "1024x768", "390x844"]:
        raise SystemExit("E_WAVE3B_G9_VIEWPORTS")
    composed = contract.get("composed_checks")
    expected_checks = [
        "tests/run_seamgrim_wave3b_foundation12_view_plan_check.py",
        "tests/run_seamgrim_wave3b_student_assignment_check.py",
        "tests/run_seamgrim_wave3b_teacher_submission_archive_check.py",
    ]
    if composed != expected_checks:
        raise SystemExit("E_WAVE3B_G9_COMPOSED_CHECKS")

    for script in [
        "tests/seamgrim_wave3b_product_quality_gate_runner.mjs",
        "tests/seamgrim_wave3b_foundation12_view_plan_runner.mjs",
        "tests/seamgrim_wave3b_student_assignment_runner.mjs",
        "tests/seamgrim_wave3b_teacher_submission_archive_runner.mjs",
    ]:
        run(["node", "--check", script], timeout=30)

    run(["python", "tests/run_pack_golden.py", "seamgrim_wave3b_product_quality_gate_v1"], timeout=120)
    run(["python", "tests/run_seamgrim_wave3b_foundation12_view_plan_check.py"], timeout=420)
    run(["python", "tests/run_seamgrim_wave3b_student_assignment_check.py"], timeout=720)
    run(["python", "tests/run_seamgrim_wave3b_teacher_submission_archive_check.py"], timeout=600)
    output = run(["node", "tests/seamgrim_wave3b_product_quality_gate_runner.mjs"], timeout=240)
    if "seamgrim_wave3b_product_quality_gate: ok" not in output:
        raise SystemExit(f"E_WAVE3B_G9_RUNNER_OUTPUT: {output}")
    print("[seamgrim-wave3b-product-quality-gate] OK behavior-closed")


if __name__ == "__main__":
    main()
