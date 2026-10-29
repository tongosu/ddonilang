#!/usr/bin/env python3
from __future__ import annotations

import json
import re
import subprocess
import sys
from copy import deepcopy
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
LESSONS_ROOT = ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons"
CARDS_PATH = LESSONS_ROOT / "lesson_cards.v0.detjson"
CROSSWALK_PATH = LESSONS_ROOT / "curriculum_crosswalk.v0.detjson"
ALLOWLIST_PATH = LESSONS_ROOT / "active_allowlist.detjson"
INDEX_PATH = LESSONS_ROOT / "index.json"
PACK_ROOT = ROOT / "pack" / "seamgrim_wave3b_foundation12_curriculum_cohort_v1"

TARGET_LESSONS = [
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
ALLOWED_VIEWS = {"graph", "space2d", "table", "text"}
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

MAEGIM_PATTERN = re.compile(r"(?m)^\s*(\w+)\s*:\s*.*?\ub9e4\uae40\s*\{")
BOIM_BLOCK_PATTERN = re.compile(r"\ubcf4\uc784\s*\{([^}]*)\}", re.S)
FIELD_PATTERN = re.compile(r"(?m)^\s*(\w+)\s*:")
DEFAULT_OBSERVATION_PATTERN = re.compile(r"(?m)^#\uae30\ubcf8\uad00\ucc30x?\s*:\s*(\w+)")


def fail(code: str, detail: str) -> int:
    print(f"{code}: {detail}", file=sys.stderr)
    return 1


def load_json(path: Path) -> object:
    return json.loads(path.read_text(encoding="utf-8"))


def run(cmd: list[str]) -> tuple[int, str]:
    proc = subprocess.run(cmd, cwd=ROOT, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    return proc.returncode, proc.stdout


def known_refs(crosswalk: dict) -> set[str]:
    refs: set[str] = set(crosswalk.get("refs", {}).keys())
    refs.update(crosswalk.get("concept_spine", {}).keys())
    refs.update(crosswalk.get("crosscutting", {}).keys())
    refs.update(crosswalk.get("misconceptions", {}).keys())
    return refs


def string_list(value: object) -> list[str]:
    return [str(item).strip() for item in value] if isinstance(value, list) else []


def extract_controls(ddn_text: str) -> set[str]:
    return set(MAEGIM_PATTERN.findall(ddn_text))


def extract_named_observations(ddn_text: str) -> set[str]:
    names: set[str] = set(DEFAULT_OBSERVATION_PATTERN.findall(ddn_text))
    for block in BOIM_BLOCK_PATTERN.findall(ddn_text):
        names.update(FIELD_PATTERN.findall(block))
    return names


def token_occurs(ddn_text: str, token: str) -> bool:
    if not token:
        return False
    return re.search(rf"(?<!\w){re.escape(token)}(?!\w)", ddn_text) is not None


def validate_bundle(cards_bundle: dict, crosswalk: dict, allowlist: dict, index: dict) -> list[str]:
    diagnostics: list[str] = []
    rows = cards_bundle.get("cards") if isinstance(cards_bundle.get("cards"), list) else []
    card_ids = [str(row.get("lesson_id", "")).strip() for row in rows]
    index_ids = {str(row.get("id", "")).strip() for row in index.get("lessons", []) if str(row.get("id", "")).strip()}
    index_by_id = {
        str(row.get("id", "")).strip(): row
        for row in index.get("lessons", [])
        if str(row.get("id", "")).strip()
    }
    allow_ids = string_list(allowlist.get("lesson_ids"))
    refs = known_refs(crosswalk)

    if cards_bundle.get("schema") != "seamgrim.lesson_card.v0":
        diagnostics.append("E_CURRICULUM_CARD_SCHEMA")
    if crosswalk.get("schema") != "seamgrim.curriculum_crosswalk.v0":
        diagnostics.append("E_CURRICULUM_CROSSWALK_SCHEMA")
    if allowlist.get("schema") != "seamgrim.active_lessons.allowlist.v1":
        diagnostics.append("E_ACTIVE_ALLOWLIST_SCHEMA")
    if len(rows) != 12 or set(card_ids) != set(TARGET_LESSONS):
        diagnostics.append("E_FOUNDATION12_CARD_TARGETS")
    if len(card_ids) != len(set(card_ids)):
        diagnostics.append("E_FOUNDATION12_DUPLICATE_CARD")
    if len(allow_ids) != len(set(allow_ids)):
        diagnostics.append("E_ACTIVE_ALLOWLIST_DUPLICATE")
    for lesson_id in allow_ids:
        if lesson_id not in index_ids:
            diagnostics.append(f"E_ACTIVE_ALLOWLIST_ORPHAN:{lesson_id}")
        if not (LESSONS_ROOT / lesson_id / "lesson.ddn").exists():
            diagnostics.append(f"E_ACTIVE_ALLOWLIST_MISSING_DDN:{lesson_id}")
    for lesson_id in TARGET_LESSONS:
        if lesson_id not in allow_ids:
            diagnostics.append(f"E_FOUNDATION12_NOT_ALLOWLISTED:{lesson_id}")

    for row in rows:
        lesson_id = str(row.get("lesson_id", "")).strip()
        if not lesson_id:
            diagnostics.append("E_CURRICULUM_CARD_MISSING_LESSON_ID")
            continue
        missing = sorted(field for field in REQUIRED_CARD_FIELDS if field not in row)
        if missing:
            diagnostics.append(f"E_CURRICULUM_CARD_MISSING_FIELD:{lesson_id}:{','.join(missing)}")
        if str(row.get("world_id", "")).strip() != lesson_id:
            diagnostics.append(f"E_CURRICULUM_CARD_WORLD_ID_MISMATCH:{lesson_id}")
        if row.get("deployment_refs", {}).get("runtime_role_toggle") is not False:
            diagnostics.append(f"E_CURRICULUM_CARD_ROLE_TOGGLE:{lesson_id}")
        if row.get("evidence", {}).get("product_allowlist") is not True:
            diagnostics.append(f"E_CURRICULUM_CARD_ALLOWLIST_EVIDENCE:{lesson_id}")
        behavior_refs = string_list(row.get("evidence", {}).get("behavior_refs"))
        if "tests/run_seamgrim_wave3b_foundation12_model_alignment_check.py" not in behavior_refs:
            diagnostics.append(f"E_CURRICULUM_CARD_BEHAVIOR_REF:{lesson_id}")
        for field in ["curriculum_refs", "concept_spine_refs", "stem_practice_refs", "crosscutting_refs", "misconception_refs"]:
            for ref in string_list(row.get(field)):
                if ref not in refs:
                    diagnostics.append(f"E_CURRICULUM_CARD_UNKNOWN_REF:{lesson_id}:{ref}")
        views = [str(row.get("primary_view", "")).strip(), *string_list(row.get("secondary_views"))]
        for view in views:
            if view not in ALLOWED_VIEWS:
                diagnostics.append(f"E_CURRICULUM_CARD_UNKNOWN_VIEW:{lesson_id}:{view}")
        required_views = set(string_list(index_by_id.get(lesson_id, {}).get("required_views")))
        if required_views and str(row.get("primary_view", "")).strip() not in required_views:
            diagnostics.append(f"E_CURRICULUM_CARD_PRIMARY_VIEW_NOT_DECLARED:{lesson_id}")
        lesson_path = ROOT / str(row.get("lesson_refs", {}).get("lesson_path", "")).replace("/", "\\")
        if not lesson_path.exists():
            diagnostics.append(f"E_CURRICULUM_CARD_MISSING_LESSON_PATH:{lesson_id}")
            continue
        ddn_text = lesson_path.read_text(encoding="utf-8")
        controls = extract_controls(ddn_text)
        for control in string_list(row.get("allowed_controls")):
            if control not in controls:
                diagnostics.append(f"E_CURRICULUM_CARD_UNKNOWN_CONTROL:{lesson_id}:{control}")
        named_observations = extract_named_observations(ddn_text)
        for observation in string_list(row.get("allowed_observations")):
            if observation not in named_observations and not token_occurs(ddn_text, observation):
                diagnostics.append(f"E_CURRICULUM_CARD_UNKNOWN_OBSERVATION:{lesson_id}:{observation}")
    return diagnostics


def expect_negative(code: str, cards: dict, crosswalk: dict, allowlist: dict, index: dict) -> str:
    diagnostics = validate_bundle(cards, crosswalk, allowlist, index)
    match = next((diag for diag in diagnostics if diag.startswith(code)), "")
    return match


def main() -> int:
    cards = load_json(CARDS_PATH)
    crosswalk = load_json(CROSSWALK_PATH)
    allowlist = load_json(ALLOWLIST_PATH)
    index = load_json(INDEX_PATH)
    for required in ["README.md", "contract.detjson", "input.ddn", "golden.jsonl"]:
        if not (PACK_ROOT / required).exists():
            return fail("E_FOUNDATION12_COHORT_PACK_MISSING", required)
    diagnostics = validate_bundle(cards, crosswalk, allowlist, index)
    if diagnostics:
        return fail("E_FOUNDATION12_COHORT_CONTRACT", json.dumps(diagnostics, ensure_ascii=False))

    damaged = deepcopy(cards)
    damaged["cards"][0].pop("prediction_prompt", None)
    if not expect_negative("E_CURRICULUM_CARD_MISSING_FIELD", damaged, crosswalk, allowlist, index):
        return fail("E_FOUNDATION12_NEGATIVE_CONTROL", "missing field was not rejected")
    damaged = deepcopy(cards)
    damaged["cards"][0]["allowed_controls"] = [*damaged["cards"][0]["allowed_controls"], "__missing_control__"]
    if not expect_negative("E_CURRICULUM_CARD_UNKNOWN_CONTROL", damaged, crosswalk, allowlist, index):
        return fail("E_FOUNDATION12_NEGATIVE_CONTROL", "unknown control was not rejected")
    damaged = deepcopy(cards)
    damaged["cards"][0]["allowed_observations"] = [*damaged["cards"][0]["allowed_observations"], "__missing_observation__"]
    if not expect_negative("E_CURRICULUM_CARD_UNKNOWN_OBSERVATION", damaged, crosswalk, allowlist, index):
        return fail("E_FOUNDATION12_NEGATIVE_CONTROL", "unknown observation was not rejected")
    damaged = deepcopy(cards)
    damaged["cards"][0]["primary_view"] = "__missing_view__"
    if not expect_negative("E_CURRICULUM_CARD_UNKNOWN_VIEW", damaged, crosswalk, allowlist, index):
        return fail("E_FOUNDATION12_NEGATIVE_CONTROL", "unknown view was not rejected")

    for cmd in [
        ["node", "--check", "tests/seamgrim_wave3b_foundation12_curriculum_cohort_runner.mjs"],
        ["node", "tests/seamgrim_wave3b_foundation12_curriculum_cohort_runner.mjs"],
    ]:
        code, output = run(cmd)
        if code != 0:
            return fail("E_FOUNDATION12_COHORT_COMMAND", f"{' '.join(cmd)}\n{output}")

    print("[seamgrim-wave3b-foundation12-curriculum-cohort-v1] OK behavior-closed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
