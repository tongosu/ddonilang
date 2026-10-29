#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import sys
import tomllib
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
LESSONS_ROOT = ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons"
PACK_ROOT = ROOT / "pack" / "seamgrim_curriculum_wave3a_baseline_v1"
BASELINE_PATH = PACK_ROOT / "baseline.detjson"

TARGET_WORLD_STARTS = [
    {
        "slot": "linear_function",
        "target_world": "rep_math_function_line_v1",
        "start_lesson_id": "rep_math_function_line_v1",
        "status": "product_active",
        "reason": "existing representative lesson with graph primary view",
    },
    {
        "slot": "pendulum",
        "target_world": "rep_phys_pendulum_wave1_v1",
        "start_lesson_id": "rep_phys_pendulum_wave1_v1",
        "status": "product_active",
        "reason": "existing representative lesson with explicit view_plan and lesson_plan",
    },
    {
        "slot": "projectile",
        "target_world": "rep_phys_projectile_xy_v1",
        "start_lesson_id": "rep_phys_projectile_xy_v1",
        "status": "product_active",
        "reason": "existing representative lesson with trajectory and world-diff evidence",
    },
    {
        "slot": "damped_oscillation",
        "target_world": "rep_phys_damped_oscillation_v1",
        "start_lesson_id": "rep_phys_damped_oscillation_v1",
        "status": "product_active",
        "reason": "promoted representative lesson with energy observations and explicit view plan",
    },
    {
        "slot": "supply_demand_tax",
        "target_world": "rep_econ_supply_demand_tax_v1",
        "start_lesson_id": "rep_econ_supply_demand_tax_v1",
        "status": "product_active",
        "reason": "existing representative lesson with market-tax graph primary view",
    },
]


def load_json(path: Path) -> object:
    with path.open("r", encoding="utf-8") as handle:
        return json.load(handle)


def read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8") if path.exists() else ""


def parse_meta(path: Path) -> tuple[dict[str, object], str | None]:
    if not path.exists():
        return {}, None
    try:
        return tomllib.loads(path.read_text(encoding="utf-8")), None
    except Exception as exc:  # Some legacy meta files are not strict TOML.
        return {}, f"{type(exc).__name__}: {exc}"


def normalize_string_list(value: object) -> list[str]:
    if isinstance(value, list):
        return [str(item) for item in value if str(item)]
    return []


def infer_prefix(lesson_id: str) -> str:
    return lesson_id.split("_", 1)[0] if "_" in lesson_id else lesson_id


def infer_grade_band(lesson_id: str, row: dict[str, object], meta: dict[str, object]) -> dict[str, str]:
    grade = str(meta.get("grade") or row.get("grade") or "").strip()
    if grade:
        return {"value": grade, "source": "lesson_metadata"}
    prefix = infer_prefix(lesson_id)
    if prefix == "elem":
        return {"value": "elementary_candidate", "source": "id_prefix_candidate"}
    if prefix == "high":
        return {"value": "high_candidate", "source": "id_prefix_candidate"}
    if prefix == "college":
        return {"value": "college_candidate", "source": "id_prefix_candidate"}
    if prefix == "ssot":
        return {"value": "ssot_edu_unspecified_candidate", "source": "id_prefix_candidate"}
    if prefix == "rep":
        return {"value": "representative_unspecified_candidate", "source": "id_prefix_candidate"}
    return {"value": "unknown", "source": "not_inferred"}


def build_inventory() -> dict[str, object]:
    index = load_json(LESSONS_ROOT / "index.json")
    index_rows = {
        str(row.get("id")): row
        for row in index.get("lessons", [])
        if isinstance(row, dict) and row.get("id")
    }
    allowlist = load_json(LESSONS_ROOT / "active_allowlist.detjson")
    active_ids = [str(item) for item in allowlist.get("lesson_ids", [])]
    active_set = set(active_ids)

    lessons: list[dict[str, object]] = []
    prefix_counts: dict[str, int] = {}
    subject_counts: dict[str, int] = {}
    grade_band_counts: dict[str, int] = {}

    for lesson_dir in sorted(path for path in LESSONS_ROOT.iterdir() if path.is_dir()):
        lesson_id = lesson_dir.name
        row = index_rows.get(lesson_id, {})
        meta, meta_error = parse_meta(lesson_dir / "meta.toml")
        prefix = infer_prefix(lesson_id)
        grade_band = infer_grade_band(lesson_id, row, meta)
        subject = str(meta.get("subject") or row.get("subject") or "unknown").strip() or "unknown"
        ssot_pack = str(meta.get("ssot_pack") or row.get("ssot_pack") or "").strip()
        required_views = normalize_string_list(meta.get("required_views") or row.get("required_views"))
        view_roles = normalize_string_list(meta.get("view_roles"))
        lesson_path = lesson_dir / "lesson.ddn"
        meta_path = lesson_dir / "meta.toml"
        text_path = lesson_dir / "text.md"
        lesson_plan_path = Path(str(meta.get("lesson_plan_path") or "")) if meta.get("lesson_plan_path") else lesson_dir / "lesson_plan.v0.toml"
        view_plan_path = Path(str(meta.get("view_plan_path") or "")) if meta.get("view_plan_path") else lesson_dir / "view_plan.detjson"
        space2d_path = lesson_dir / "space2d.json"
        representation_pair_path = Path(str(meta.get("representation_pair_path") or "")) if meta.get("representation_pair_path") else lesson_dir / "representation_pair.detjson"
        world_diff_path = Path(str(meta.get("world_diff_path") or "")) if meta.get("world_diff_path") else lesson_dir / "world_diff_gravity_half.detjson"
        inputs_dir = lesson_dir / "inputs"
        pack_path = ROOT / "pack" / ssot_pack if ssot_pack else None
        row_out = {
            "id": lesson_id,
            "prefix": prefix,
            "title": str(meta.get("title") or row.get("title") or ""),
            "description_present": bool(str(meta.get("description") or row.get("description") or "").strip()),
            "subject": subject,
            "grade_band_candidate": grade_band["value"],
            "grade_band_candidate_source": grade_band["source"],
            "level": str(meta.get("level") or row.get("level") or ""),
            "required_views": required_views,
            "primary_view": str(meta.get("primary_view") or ""),
            "view_roles": view_roles,
            "goals_count": len(normalize_string_list(meta.get("goals") or row.get("goals"))),
            "files": {
                "lesson_ddn": lesson_path.exists(),
                "meta_toml": meta_path.exists(),
                "text_md": text_path.exists(),
                "lesson_plan_v0": (ROOT / lesson_plan_path).exists() if not lesson_plan_path.is_absolute() else lesson_plan_path.exists(),
                "view_plan": (ROOT / view_plan_path).exists() if not view_plan_path.is_absolute() else view_plan_path.exists(),
                "space2d": space2d_path.exists(),
                "representation_pair": (ROOT / representation_pair_path).exists() if not representation_pair_path.is_absolute() else representation_pair_path.exists(),
                "world_diff": (ROOT / world_diff_path).exists() if not world_diff_path.is_absolute() else world_diff_path.exists(),
                "inputs_count": len(list(inputs_dir.glob("*.ddn"))) if inputs_dir.exists() else 0,
            },
            "product_allowlist": lesson_id in active_set,
            "index_row_present": lesson_id in index_rows,
            "execution_surface": "ddn_lesson" if lesson_path.exists() else "metadata_only",
            "pack_evidence": {
                "ssot_pack": ssot_pack,
                "ssot_pack_exists": bool(pack_path and pack_path.exists()),
            },
            "meta_parse_error": meta_error or "",
        }
        lessons.append(row_out)
        prefix_counts[prefix] = prefix_counts.get(prefix, 0) + 1
        subject_counts[subject] = subject_counts.get(subject, 0) + 1
        grade_band_counts[grade_band["value"]] = grade_band_counts.get(grade_band["value"], 0) + 1

    lessons_by_id = {row["id"]: row for row in lessons}
    representative_start_points = []
    for start in TARGET_WORLD_STARTS:
        lesson = lessons_by_id.get(start["start_lesson_id"], {})
        representative_start_points.append({
            **start,
            "exists": bool(lesson),
            "product_allowlist": bool(lesson.get("product_allowlist", False)),
            "primary_view": str(lesson.get("primary_view", "")),
            "required_views": lesson.get("required_views", []),
            "view_plan_present": bool(lesson.get("files", {}).get("view_plan", False)) if isinstance(lesson.get("files"), dict) else False,
            "lesson_plan_present": bool(lesson.get("files", {}).get("lesson_plan_v0", False)) if isinstance(lesson.get("files"), dict) else False,
            "current_gaps": current_gaps_for_slot(start["slot"], lesson),
        })

    return {
        "schema": "seamgrim.curriculum_wave3a.baseline.v1",
        "source": {
            "lessons_root": "solutions/seamgrim_ui_mvp/lessons",
            "lesson_index": "solutions/seamgrim_ui_mvp/lessons/index.json",
            "active_allowlist": "solutions/seamgrim_ui_mvp/lessons/active_allowlist.detjson",
            "masterplan": "docs/context/roadmap/SEAMGRIM_DDONIRANG_K12_UNIVERSITY_STEM_CURRICULUM_MASTERPLAN_V1_20260712.md",
        },
        "counts": {
            "lesson_directories": len(lessons),
            "index_rows": len(index_rows),
            "active_allowlist": len(active_ids),
            "prefix": dict(sorted(prefix_counts.items())),
            "subject": dict(sorted(subject_counts.items())),
            "grade_band_candidate": dict(sorted(grade_band_counts.items())),
        },
        "claims_boundary": {
            "quantity_is_not_closure": True,
            "grade_band_is_candidate_unless_lesson_metadata": True,
            "curriculum_standard_alignment_claim": False,
            "global_curriculum_completion_claim": False,
            "state_hash_formula_change": False,
            "ssot_edit": False,
        },
        "representative_start_points": representative_start_points,
        "lessons": lessons,
    }


def current_gaps_for_slot(slot: str, lesson: dict[str, object]) -> list[str]:
    files = lesson.get("files", {}) if isinstance(lesson.get("files"), dict) else {}
    gaps: list[str] = []
    if not lesson:
        return ["source_lesson_absent"]
    if not lesson.get("product_allowlist", False):
        gaps.append("not_active_representative_lesson")
    if not files.get("lesson_plan_v0", False):
        gaps.append("lesson_plan_v0_missing")
    if slot in {"pendulum", "projectile", "damped_oscillation"} and not files.get("view_plan", False):
        gaps.append("explicit_view_plan_missing")
    gaps.extend([
        "prediction_artifact_pending",
        "explanation_artifact_pending",
        "saved_project_lineage_pending",
        "teacher_review_packet_pending",
    ])
    return gaps


def stable_json_bytes(data: object) -> bytes:
    return (json.dumps(data, ensure_ascii=False, indent=2, sort_keys=True) + "\n").encode("utf-8")


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}", file=sys.stderr)
    return 1


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--write", action="store_true", help="rewrite the baseline fixture")
    args = parser.parse_args()

    baseline = build_inventory()
    expected_bytes = stable_json_bytes(baseline)
    if args.write:
        PACK_ROOT.mkdir(parents=True, exist_ok=True)
        BASELINE_PATH.write_bytes(expected_bytes)
        print(f"wrote {BASELINE_PATH.relative_to(ROOT)}")
        return 0

    if not BASELINE_PATH.exists():
        return fail("E_SEAMGRIM_W3A_BASELINE_MISSING", str(BASELINE_PATH.relative_to(ROOT)))
    actual_bytes = BASELINE_PATH.read_bytes()
    if actual_bytes != expected_bytes:
        return fail("E_SEAMGRIM_W3A_BASELINE_DRIFT", "run with --write after auditing intentional changes")

    data = json.loads(actual_bytes.decode("utf-8"))
    counts = data.get("counts", {})
    if counts.get("lesson_directories") != 218:
        return fail("E_SEAMGRIM_W3A_LESSON_COUNT", str(counts.get("lesson_directories")))
    if counts.get("active_allowlist") != 17:
        return fail("E_SEAMGRIM_W3A_ALLOWLIST_COUNT", str(counts.get("active_allowlist")))
    starts = data.get("representative_start_points", [])
    if len(starts) != 5 or any(not row.get("exists") for row in starts):
        return fail("E_SEAMGRIM_W3A_START_POINTS", json.dumps(starts, ensure_ascii=False))
    damping = next((row for row in starts if row.get("slot") == "damped_oscillation"), None)
    if not damping or damping.get("status") != "product_active" or damping.get("product_allowlist") is not True:
        return fail("E_SEAMGRIM_W3A_DAMPING_BOUNDARY", json.dumps(damping, ensure_ascii=False))
    boundary = data.get("claims_boundary", {})
    if boundary.get("curriculum_standard_alignment_claim") is not False or boundary.get("global_curriculum_completion_claim") is not False:
        return fail("E_SEAMGRIM_W3A_OVERCLAIM_BOUNDARY", json.dumps(boundary, ensure_ascii=False))
    print("[seamgrim-curriculum-wave3a-baseline-v1] OK inventory-closed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
