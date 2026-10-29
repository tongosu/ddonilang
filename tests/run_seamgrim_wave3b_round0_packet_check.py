#!/usr/bin/env python3
"""Gate for Wave3B G10 Round 0 packet and integration boundary."""

from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_wave3b_foundation12_round0_packet_v1"
REPORT = ROOT / "docs" / "context" / "reports" / "SEAMGRIM_WAVE3B_FOUNDATION12_ROUND0_PACKET_V1_20260712.md"

LESSON_IDS = [
    "elem_math_number_line",
    "elem_math_fraction_line",
    "elem_math_rectangle_area",
    "elem_science_daily_temperature",
    "elem_econ_allowance_budget",
    "elem_econ_store_stock",
    "rep_math_function_line_v1",
    "rep_phys_pendulum_wave1_v1",
    "rep_phys_projectile_xy_v1",
    "rep_econ_supply_demand_tax_v1",
    "rep_phys_damped_oscillation_v1",
    "high_math_derivative",
]

COMMITS = [
    "b7e2a22",
    "7bfc370",
    "eae8847",
    "a78f847",
    "e7039e0",
    "10bb62d",
    "5383604",
    "2fb2559",
    "dfc3695",
    "c6b604c",
    "b670531",
]


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
        raise SystemExit(f"E_WAVE3B_G10_COMMAND: {' '.join(cmd)}\n{detail}")
    return ((proc.stdout or "") + (proc.stderr or "")).strip()


def main() -> None:
    contract = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    required = {
        "product_behavior_claim": False,
        "round0_packet_required": True,
        "foundation_lesson_count": 12,
        "gate_commit_count": 11,
        "classroom_exchange_schema": "seamgrim.classroom_exchange.v0",
        "teacher_student_teacher_loop": True,
        "round0_teacher_self_review": True,
        "external_student_pilot_prep": True,
        "independent_review_request": True,
        "no_runtime_role_toggle": True,
        "docs_ssot_change": False,
        "parser_change": False,
        "state_hash_formula_change": False,
        "education_effect_claim": False,
        "idc_ready_claim": False,
        "account_identity_claim": False,
        "cloud_sync_claim": False,
        "public_upload_claim": False,
        "korean_manim_overall_completion_claim": False,
        "ai_auto_assessment_claim": False,
    }
    for key, expected in required.items():
        actual = contract.get(key)
        if actual != expected:
            raise SystemExit(f"E_WAVE3B_G10_CONTRACT_{key}: expected {expected!r}, got {actual!r}")
    if contract.get("required_gates") != [*[f"G{i}" for i in range(9)], "G8b", "G9"]:
        raise SystemExit("E_WAVE3B_G10_GATE_LIST")
    if contract.get("report_path") != "docs/context/reports/SEAMGRIM_WAVE3B_FOUNDATION12_ROUND0_PACKET_V1_20260712.md":
        raise SystemExit("E_WAVE3B_G10_REPORT_PATH")

    text = REPORT.read_text(encoding="utf-8")
    for commit in COMMITS:
        if commit not in text:
            raise SystemExit(f"E_WAVE3B_G10_MISSING_COMMIT: {commit}")
    for lesson_id in LESSON_IDS:
        if f"`{lesson_id}`" not in text:
            raise SystemExit(f"E_WAVE3B_G10_MISSING_LESSON: {lesson_id}")
    for token in [
        "Round 0 Teacher Self-Review",
        "External Student Pilot Prep",
        "Independent Review Request",
        "Not claimed",
        "education-effect",
        "Korean Manim overall completion",
    ]:
        if token not in text:
            raise SystemExit(f"E_WAVE3B_G10_MISSING_TOKEN: {token}")

    run(["python", "tests/run_pack_golden.py", "seamgrim_wave3b_foundation12_round0_packet_v1"], timeout=120)
    print("[seamgrim-wave3b-round0-packet] OK docs-closed")


if __name__ == "__main__":
    main()
