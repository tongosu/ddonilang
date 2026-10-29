#!/usr/bin/env python3
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
LESSONS_ROOT = ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons"
PACK = ROOT / "pack" / "seamgrim_wave3b_foundation12_semantic_view_plans_v1"
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


def load_json(path: Path) -> Any:
    return json.loads(path.read_text(encoding="utf-8"))


def run(cmd: list[str], timeout: int = 300) -> tuple[int, str]:
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


def check_static() -> int:
    contract = load_json(PACK / "contract.detjson")
    if contract.get("world_count") != 12 or contract.get("schema") != "seamgrim.wave3b.foundation12.semantic_view_plans.contract.v1":
        return fail("E_WAVE3B_G3_CONTRACT", json.dumps(contract, ensure_ascii=False))
    for key in ("state_hash_formula_change", "new_ddn_syntax", "docs_ssot_edit", "world_state_stores_view_objects"):
        if contract.get(key) is not False:
            return fail("E_WAVE3B_G3_BOUNDARY", f"{key}={contract.get(key)!r}")
    missing = []
    for lesson_id in TARGETS:
        lesson_dir = LESSONS_ROOT / lesson_id
        plan_path = lesson_dir / "view_plan.detjson"
        meta_path = lesson_dir / "meta.toml"
        if not plan_path.exists():
            missing.append(f"{lesson_id}/view_plan.detjson")
            continue
        meta_text = meta_path.read_text(encoding="utf-8")
        if f"{lesson_id}/view_plan.detjson" not in meta_text:
            return fail("E_WAVE3B_G3_META_LINK", lesson_id)
        plan = load_json(plan_path)
        if plan.get("schema") != "seamgrim.view_plan.v1" or plan.get("binding_mode") != "explicit":
            return fail("E_WAVE3B_G3_VIEW_PLAN_SCHEMA", lesson_id)
        anchors = [row for row in plan.get("anchors", []) if isinstance(row, dict)]
        if len(anchors) < 2:
            return fail("E_WAVE3B_G3_ANCHOR_COUNT", lesson_id)
        lesson_text = (lesson_dir / "lesson.ddn").read_text(encoding="utf-8")
        for anchor in anchors:
            if not str(anchor.get("unit", "")).strip():
                return fail("E_WAVE3B_G3_ANCHOR_UNIT", f"{lesson_id}:{anchor.get('anchor_id')}")
            if not (str(anchor.get("accessibility_name", "")).strip() or str(anchor.get("label", "")).strip()):
                return fail("E_WAVE3B_G3_ANCHOR_A11Y", f"{lesson_id}:{anchor.get('anchor_id')}")
            provenance = anchor.get("provenance", {})
            if f"/lessons/{lesson_id}/lesson.ddn" not in str(provenance.get("path", "")):
                return fail("E_WAVE3B_G3_ANCHOR_PATH", f"{lesson_id}:{anchor.get('anchor_id')}")
            span = str(provenance.get("source_span", "")).strip()
            span_parts = [part.strip() for part in span.split(";") if part.strip()]
            if not span_parts or any(part not in lesson_text for part in span_parts):
                return fail("E_WAVE3B_G3_ANCHOR_SPAN", f"{lesson_id}:{anchor.get('anchor_id')}:{span}")
        if plan.get("hash_policy", {}).get("state_hash_participation") is not False:
            return fail("E_WAVE3B_G3_HASH_POLICY", lesson_id)
    if missing:
        return fail("E_WAVE3B_G3_MISSING", repr(missing))
    return 0


def main() -> int:
    rc = check_static()
    if rc:
        return rc
    commands = [
        ["node", "--check", "solutions/seamgrim_ui_mvp/ui/view_plan_runtime.js"],
        ["node", "--check", "solutions/seamgrim_ui_mvp/ui/screens/run.js"],
        ["node", "--check", "tests/seamgrim_wave3b_foundation12_semantic_view_plans_runner.mjs"],
        ["python", "tests/run_pack_golden.py", "seamgrim_wave3b_foundation12_semantic_view_plans_v1"],
        ["node", "tests/seamgrim_wave3b_foundation12_semantic_view_plans_runner.mjs"],
    ]
    for cmd in commands:
        code, output = run(cmd, timeout=420)
        if code != 0:
            return fail("E_WAVE3B_G3_COMMAND", f"{' '.join(cmd)}\n{output}")
    print("[seamgrim-wave3b-foundation12-semantic-view-plans] OK behavior-closed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
