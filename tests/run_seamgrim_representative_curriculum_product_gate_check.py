#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")
    sys.stderr.reconfigure(encoding="utf-8", errors="replace")

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_representative_curriculum_product_gate_v1"
LESSONS = [
    "rep_math_function_line_v1",
    "rep_phys_pendulum_wave1_v1",
    "rep_phys_projectile_xy_v1",
    "rep_phys_damped_oscillation_v1",
    "rep_econ_supply_demand_tax_v1",
]
REQUIRED_ARTIFACTS = [
    "lesson_card.v0",
    "seamgrim.curriculum_inquiry_artifact.v0",
    "seamgrim.curriculum_artifact_bundle.v0",
    "seamgrim.curriculum_model_comparison.v0",
    "seamgrim.curriculum_semantic_scene.v0",
]


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


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


def check_files_and_contract() -> int:
    required = [
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        ROOT / "tests" / "seamgrim_representative_curriculum_product_gate_runner.mjs",
        ROOT / "tests" / "run_seamgrim_curriculum_cards_check.py",
        ROOT / "tests" / "run_seamgrim_curriculum_workflow_three_worlds_check.py",
        ROOT / "tests" / "run_seamgrim_projectile_damping_promotion_check.py",
        ROOT / "tests" / "run_seamgrim_curriculum_artifact_bundle_check.py",
        ROOT / "tests" / "run_seamgrim_model_comparison_teaching_check.py",
        ROOT / "tests" / "run_seamgrim_semantic_scenes_five_lessons_check.py",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_REP_CURRICULUM_GATE_MISSING", repr(missing))
    contract = json.loads(read(PACK / "contract.detjson"))
    if contract.get("schema") != "seamgrim.representative_curriculum_product_gate.contract.v1":
        return fail("E_REP_CURRICULUM_GATE_SCHEMA", str(contract.get("schema")))
    if contract.get("lesson_ids") != LESSONS:
        return fail("E_REP_CURRICULUM_GATE_LESSONS", repr(contract.get("lesson_ids")))
    if contract.get("required_product_artifacts") != REQUIRED_ARTIFACTS:
        return fail("E_REP_CURRICULUM_GATE_ARTIFACTS", repr(contract.get("required_product_artifacts")))
    boundary = contract.get("claim_boundary", {})
    for key in [
        "canonical_curriculum_alignment",
        "external_student_success_claim",
        "korean_manim_complete_claim",
        "new_ddn_syntax",
        "state_hash_formula_change",
        "runtime_role_toggle",
    ]:
        if boundary.get(key) is not False:
            return fail("E_REP_CURRICULUM_GATE_BOUNDARY", f"{key}={boundary.get(key)!r}")
    return 0


def run_gate(cmd: list[str], code: str, timeout: int) -> int:
    proc = run(cmd, timeout=timeout)
    if proc.returncode != 0:
        return fail(code, f"{' '.join(cmd)}\n{proc.stdout.strip()}")
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_REP_CURRICULUM_GATE_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_REP_CURRICULUM_GATE_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    rc = check_files_and_contract()
    if rc:
        return rc
    gates = [
        (["node", "--check", "tests/seamgrim_representative_curriculum_product_gate_runner.mjs"], "E_REP_CURRICULUM_GATE_RUNNER_NODE", 60),
        (["node", "tests/seamgrim_representative_curriculum_product_gate_runner.mjs"], "E_REP_CURRICULUM_GATE_BROWSER", 420),
        (["python", "tests/run_pack_golden.py", "seamgrim_representative_curriculum_product_gate_v1"], "E_REP_CURRICULUM_GATE_GOLDEN", 300),
        (["python", "tests/run_seamgrim_curriculum_cards_check.py"], "E_REP_CURRICULUM_GATE_G1", 240),
        (["python", "tests/run_seamgrim_curriculum_workflow_three_worlds_check.py"], "E_REP_CURRICULUM_GATE_G2", 420),
        (["python", "tests/run_seamgrim_projectile_damping_promotion_check.py"], "E_REP_CURRICULUM_GATE_G3", 420),
        (["python", "tests/run_seamgrim_curriculum_artifact_bundle_check.py"], "E_REP_CURRICULUM_GATE_G4", 480),
        (["python", "tests/run_seamgrim_model_comparison_teaching_check.py"], "E_REP_CURRICULUM_GATE_G5", 420),
        (["python", "tests/run_seamgrim_semantic_scenes_five_lessons_check.py"], "E_REP_CURRICULUM_GATE_G6", 480),
    ]
    for cmd, code, timeout in gates:
        rc = run_gate(cmd, code, timeout)
        if rc:
            return rc
    rc = check_docs_ssot_clean()
    if rc:
        return rc
    print("[seamgrim-representative-curriculum-product-gate-v1] OK behavior-closed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
