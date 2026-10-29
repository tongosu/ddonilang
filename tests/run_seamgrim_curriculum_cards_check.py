#!/usr/bin/env python3
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CARDS_PATH = ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons" / "lesson_cards.v0.detjson"
CROSSWALK_PATH = ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons" / "curriculum_crosswalk.v0.detjson"

REQUIRED_CARD_FIELDS = {
    "lesson_id",
    "world_id",
    "grade_band",
    "subject_refs",
    "curriculum_refs",
    "concept_spine_refs",
    "stem_practice_refs",
    "crosscutting_refs",
    "primary_view",
    "secondary_views",
    "allowed_controls",
    "allowed_observations",
    "prediction_prompt",
    "inquiry_prompt",
    "misconception_refs",
    "lesson_refs",
    "view_refs",
    "deployment_refs",
    "lineage",
    "evidence",
    "status_axes",
}


def fail(code: str, detail: str) -> int:
    print(f"{code}: {detail}", file=sys.stderr)
    return 1


def run(cmd: list[str]) -> tuple[int, str]:
    proc = subprocess.run(cmd, cwd=ROOT, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    return proc.returncode, proc.stdout


def main() -> int:
    cards = json.loads(CARDS_PATH.read_text(encoding="utf-8"))
    crosswalk = json.loads(CROSSWALK_PATH.read_text(encoding="utf-8"))
    if cards.get("schema") != "seamgrim.lesson_card.v0":
        return fail("E_CURRICULUM_CARD_SCHEMA", str(cards.get("schema")))
    if crosswalk.get("schema") != "seamgrim.curriculum_crosswalk.v0":
        return fail("E_CURRICULUM_CROSSWALK_SCHEMA", str(crosswalk.get("schema")))
    card_rows = cards.get("cards")
    if not isinstance(card_rows, list) or len(card_rows) != 12:
        return fail("E_CURRICULUM_CARD_COUNT", str(len(card_rows) if isinstance(card_rows, list) else type(card_rows).__name__))
    target_ids = {
        "elem_math_number_line",
        "elem_math_fraction_line",
        "elem_math_rectangle_area",
        "elem_science_daily_temperature",
        "elem_econ_allowance_budget",
        "elem_econ_store_stock",
        "rep_math_function_line_v1",
        "rep_phys_pendulum_wave1_v1",
        "rep_phys_projectile_xy_v1",
        "rep_phys_damped_oscillation_v1",
        "rep_econ_supply_demand_tax_v1",
        "high_math_derivative",
    }
    if {str(row.get("lesson_id")) for row in card_rows} != target_ids:
        return fail("E_CURRICULUM_CARD_TARGETS", json.dumps([row.get("lesson_id") for row in card_rows], ensure_ascii=False))
    for row in card_rows:
        missing = sorted(field for field in REQUIRED_CARD_FIELDS if field not in row)
        if missing:
            return fail("E_CURRICULUM_CARD_MISSING_FIELD", f"{row.get('lesson_id')}: {','.join(missing)}")
        if row.get("deployment_refs", {}).get("runtime_role_toggle") is not False:
            return fail("E_CURRICULUM_CARD_ROLE_TOGGLE", str(row.get("lesson_id")))
    if cards.get("claim_boundary", {}).get("canonical_curriculum_alignment") is not False:
        return fail("E_CURRICULUM_CARD_OVERCLAIM", "canonical alignment must stay false")

    for cmd in [
        ["node", "--check", "solutions/seamgrim_ui_mvp/ui/curriculum_cards.js"],
        ["node", "--check", "tests/seamgrim_curriculum_cards_runner.mjs"],
        ["node", "tests/seamgrim_curriculum_cards_runner.mjs"],
    ]:
        code, output = run(cmd)
        if code != 0:
            return fail("E_CURRICULUM_CARD_COMMAND", f"{' '.join(cmd)}\n{output}")
    print("[seamgrim-curriculum-cards-v0] OK behavior-closed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
