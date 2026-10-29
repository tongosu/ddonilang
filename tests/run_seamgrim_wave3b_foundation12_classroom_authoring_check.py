#!/usr/bin/env python3
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_wave3b_foundation12_classroom_authoring_v1"
TARGETS = [
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


def fail(code: str, detail: str) -> int:
    print(f"{code}: {detail}", file=sys.stderr)
    return 1


def run(cmd: list[str], timeout: int = 420) -> tuple[int, str]:
    proc = subprocess.run(
        cmd,
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        timeout=timeout,
    )
    return proc.returncode, proc.stdout


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def check_static() -> int:
    contract = load_json(PACK / "contract.detjson")
    if contract.get("schema") != "seamgrim.wave3b.foundation12.classroom_authoring.contract.v1":
        return fail("E_WAVE3B_G4_CONTRACT_SCHEMA", json.dumps(contract, ensure_ascii=False))
    if contract.get("world_count") != len(TARGETS):
        return fail("E_WAVE3B_G4_WORLD_COUNT", str(contract.get("world_count")))
    for key in [
        "requires_foundation_world_select",
        "requires_root_questions",
        "requires_allowed_controls",
        "requires_primary_secondary_view",
        "requires_misconception",
        "requires_reading_check",
        "requires_rubric",
        "requires_deployment_profile_lock",
        "requires_list_code_roundtrip",
        "requires_save_reopen_roundtrip",
        "ddn_world_source_separate",
    ]:
        if contract.get(key) is not True:
            return fail("E_WAVE3B_G4_REQUIRED_FALSE", f"{key}={contract.get(key)!r}")
    for key in ["runtime_role_toggle_restored", "new_ddn_syntax", "state_hash_formula_change", "docs_ssot_edit"]:
        if contract.get(key) is not False:
            return fail("E_WAVE3B_G4_BOUNDARY", f"{key}={contract.get(key)!r}")
    return 0


def main() -> int:
    rc = check_static()
    if rc:
        return rc
    commands = [
        ["node", "--check", "solutions/seamgrim_ui_mvp/ui/app.js"],
        ["node", "--check", "solutions/seamgrim_ui_mvp/ui/screens/editor.js"],
        ["node", "--check", "solutions/seamgrim_ui_mvp/ui/lesson_plan_v0.js"],
        ["node", "--check", "tests/seamgrim_wave3b_foundation12_classroom_authoring_runner.mjs"],
        ["python", "tests/run_pack_golden.py", "seamgrim_wave3b_foundation12_classroom_authoring_v1"],
        ["node", "tests/seamgrim_wave3b_foundation12_classroom_authoring_runner.mjs"],
    ]
    for cmd in commands:
        code, output = run(cmd, timeout=720)
        if code:
            return fail("E_WAVE3B_G4_COMMAND", f"{' '.join(cmd)}\n{output}")
    print("[seamgrim-wave3b-foundation12-classroom-authoring] OK behavior-closed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
