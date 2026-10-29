#!/usr/bin/env python
from __future__ import annotations

import subprocess
import sys
from pathlib import Path


if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")
    sys.stderr.reconfigure(encoding="utf-8", errors="replace")

ROOT = Path(__file__).resolve().parents[1]
REPORT = ROOT / "docs" / "reports" / "impl" / "SEAMGRIM_WAVE3A_FIVE_LESSON_CURRICULUM_PILOT_PACKET_V1_20260712.md"
LESSONS = [
    "rep_math_function_line_v1",
    "rep_phys_pendulum_wave1_v1",
    "rep_phys_projectile_xy_v1",
    "rep_phys_damped_oscillation_v1",
    "rep_econ_supply_demand_tax_v1",
]
REQUIRED_HEADINGS = [
    "## 범위",
    "## 교사 검수표",
    "## 20분 운영안",
    "## 45분 운영안",
    "## 90분 운영안",
    "## 예상 오개념과 질문",
    "## 학생 읽기 시험",
    "## 접근성 확인표",
    "## Pilot 기록 양식",
    "## 닫힘 등급",
    "## 알려진 한계",
    "## 검증 명령",
]
BOUNDARY_TOKENS = [
    "canonical_curriculum_alignment=false",
    "external_student_success_claim=false",
    "korean_manim_complete_claim=false",
    "new_ddn_syntax=false",
    "state_hash_formula_change=false",
    "runtime_role_toggle=false",
    "닫힘-실행",
    "닫힘-보기",
    "닫힘-저장",
    "닫힘-문서",
    "닫힘-교육 아님",
]


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def run(cmd: list[str], *, timeout: int = 300) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        timeout=timeout,
    )


def check_report() -> int:
    if not REPORT.exists():
        return fail("E_WAVE3A_PILOT_PACKET_MISSING", str(REPORT.relative_to(ROOT)))
    text = REPORT.read_text(encoding="utf-8")
    for heading in REQUIRED_HEADINGS:
        if heading not in text:
            return fail("E_WAVE3A_PILOT_PACKET_HEADING", heading)
    for lesson_id in LESSONS:
        if text.count(f"`{lesson_id}`") < 3:
            return fail("E_WAVE3A_PILOT_PACKET_LESSON", lesson_id)
    for token in BOUNDARY_TOKENS:
        if token not in text:
            return fail("E_WAVE3A_PILOT_PACKET_BOUNDARY", token)
    for command in [
        "python tests/run_seamgrim_curriculum_wave3a_baseline_check.py",
        "python tests/run_seamgrim_projectile_damping_promotion_check.py",
        "python tests/run_seamgrim_semantic_scenes_five_lessons_check.py",
        "python tests/run_seamgrim_representative_curriculum_product_gate_check.py",
        "python tests/run_seamgrim_wave3a_representative_viewport_gate_check.py",
        "python tests/run_seamgrim_wave3a_pilot_packet_check.py",
        "python tests/run_ci_sanity_gate.py --profile core_lang",
        "git status --short -- docs/ssot",
    ]:
        if command not in text:
            return fail("E_WAVE3A_PILOT_PACKET_COMMAND", command)
    return 0


def run_gate(cmd: list[str], code: str, timeout: int) -> int:
    proc = run(cmd, timeout=timeout)
    if proc.returncode != 0:
        return fail(code, f"{' '.join(cmd)}\n{proc.stdout.strip()}")
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_WAVE3A_PILOT_PACKET_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_WAVE3A_PILOT_PACKET_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    rc = check_report()
    if rc:
        return rc
    gates = [
        (["python", "tests/run_seamgrim_curriculum_wave3a_baseline_check.py"], "E_WAVE3A_PILOT_PACKET_BASELINE", 300),
        (["python", "tests/run_seamgrim_projectile_damping_promotion_check.py"], "E_WAVE3A_PILOT_PACKET_DAMPING", 1200),
        (["python", "tests/run_seamgrim_semantic_scenes_five_lessons_check.py"], "E_WAVE3A_PILOT_PACKET_SEMANTIC", 1200),
        (["python", "tests/run_pack_golden.py", "seamgrim_representative_curriculum_product_gate_v1"], "E_WAVE3A_PILOT_PACKET_GOLDEN", 300),
        (["python", "tests/run_seamgrim_wave3a_representative_viewport_gate_check.py"], "E_WAVE3A_PILOT_PACKET_VIEWPORT", 1200),
        (["python", "tests/run_seamgrim_representative_curriculum_product_gate_check.py"], "E_WAVE3A_PILOT_PACKET_PRODUCT_GATE", 1200),
    ]
    for cmd, code, timeout in gates:
        rc = run_gate(cmd, code, timeout)
        if rc:
            return rc
    rc = check_docs_ssot_clean()
    if rc:
        return rc
    print("[seamgrim-wave3a-pilot-packet-v1] OK docs-closed with product-gate evidence")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
