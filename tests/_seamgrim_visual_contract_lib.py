from __future__ import annotations

import json
import re
from pathlib import Path


CORE_REWRITE_LESSONS_STATIC = [
    "physics_motion",
    "physics_projectile",
    "physics_circuit",
    "high_physics_wave",
    "college_physics_harmonic",
    "high_physics_damped",
    "high_physics_energy",
    "college_physics_orbit",
    "college_physics_thermal",
    "edu_pilot_phys_econ_lesson_phys_01",
]

SHAPE_BLOCK_RE = re.compile(r"^\s*(보개|모양)\s*:?\s*\{", re.MULTILINE)
LEGACY_SHAPE_MARKER_RE = re.compile(r'"space2d(\.shape)?"\s+보여주기\.', re.IGNORECASE)
SERIES_MARKER_RE = re.compile(r'"series:[^"]+"')


def validate_visual_contract(raw: bytes, text: str) -> str | None:
    if raw.startswith(b"\xef\xbb\xbf"):
        return "utf8_bom_found"
    if not SHAPE_BLOCK_RE.search(text):
        return "shape_block_missing"
    if LEGACY_SHAPE_MARKER_RE.search(text):
        return "legacy_shape_marker_found"
    if "#기본관찰x:" not in text:
        return "default_x_meta_missing"
    if "#기본관찰:" not in text:
        return "default_y_meta_missing"
    if not SERIES_MARKER_RE.search(text):
        return "series_marker_missing"
    return None


def _collect_rewrite_lesson_ids(base: Path) -> tuple[list[str], str | None]:
    discovered_phys = sorted(
        path.name for path in base.iterdir() if path.is_dir() and path.name.startswith("ssot_edu_phys_p")
    )
    if not discovered_phys:
        return [], "ssot_phys_lessons_missing"

    discovered_econ = sorted(
        path.name for path in base.iterdir() if path.is_dir() and path.name.startswith("ssot_edu_econ_")
    )
    if not discovered_econ:
        return [], "ssot_econ_lessons_missing"

    discovered_s = sorted(path.name for path in base.iterdir() if path.is_dir() and path.name.startswith("ssot_edu_s"))
    if not discovered_s:
        return [], "ssot_s_lessons_missing"

    ids: list[str] = []
    seen: set[str] = set()
    for lesson_id in CORE_REWRITE_LESSONS_STATIC + discovered_phys + discovered_econ + discovered_s:
        if lesson_id in seen:
            continue
        ids.append(lesson_id)
        seen.add(lesson_id)
    return ids, None


def run_rewrite_core_shape_policy(root: Path) -> tuple[str | None, int]:
    base = root / "solutions" / "seamgrim_ui_mvp" / "lessons_rewrite_v1"
    if not base.exists():
        return f"rewrite_base_missing:{base.as_posix()}", 0

    lesson_ids, collect_error = _collect_rewrite_lesson_ids(base)
    if collect_error:
        return collect_error, 0

    checked = 0
    for lesson_id in lesson_ids:
        lesson = base / lesson_id / "lesson.ddn"
        if not lesson.exists():
            return f"lesson_missing:{lesson_id}:{lesson.as_posix()}", checked
        raw = lesson.read_bytes()
        try:
            text = raw.decode("utf-8")
        except UnicodeDecodeError as exc:
            return f"utf8_decode_failed:{lesson_id}:{exc}", checked
        code = validate_visual_contract(raw, text)
        if code:
            return f"{code}:{lesson_id}", checked
        checked += 1

    return None, checked


def run_seed_shape_policy(root: Path) -> tuple[str | None, int]:
    meta_detail, _meta_count = run_seed_meta_files_policy(root)
    if meta_detail:
        return meta_detail, 0

    manifest = root / "solutions" / "seamgrim_ui_mvp" / "seed_lessons_v1" / "seed_manifest.detjson"
    if not manifest.exists():
        return f"manifest_missing:{manifest.as_posix()}", 0

    try:
        payload = json.loads(manifest.read_text(encoding="utf-8"))
    except Exception as exc:  # pragma: no cover - defensive
        return f"manifest_parse_failed:{exc}", 0

    seeds = payload.get("seeds")
    if not isinstance(seeds, list) or not seeds:
        return "seed_list_missing", 0

    checked = 0
    for row in seeds:
        if not isinstance(row, dict):
            return "seed_row_invalid", checked
        seed_id = str(row.get("seed_id", "")).strip()
        if not seed_id:
            return "seed_id_missing", checked

        lesson_rel = str(row.get("lesson_ddn", "")).strip()
        if not lesson_rel:
            lesson_rel = f"solutions/seamgrim_ui_mvp/seed_lessons_v1/{seed_id}/lesson.ddn"
        lesson_path = root / lesson_rel
        if not lesson_path.exists():
            return f"lesson_missing:{seed_id}:{lesson_rel}", checked

        raw = lesson_path.read_bytes()
        try:
            text = raw.decode("utf-8")
        except UnicodeDecodeError as exc:
            return f"utf8_decode_failed:{seed_id}:{lesson_rel}:{exc}", checked
        code = validate_visual_contract(raw, text)
        if code:
            return f"{code}:{seed_id}:{lesson_rel}", checked
        checked += 1

    return None, checked


def run_seed_meta_files_policy(root: Path) -> tuple[str | None, int]:
    seed_root = root / "solutions" / "seamgrim_ui_mvp" / "seed_lessons_v1"
    if not seed_root.exists():
        return f"seed_root_missing:{seed_root.as_posix()}", 0

    lesson_dirs = sorted([path for path in seed_root.iterdir() if path.is_dir()])
    if not lesson_dirs:
        return "seed_lessons_empty", 0

    missing_meta: list[str] = []
    for lesson_dir in lesson_dirs:
        meta = lesson_dir / "meta.toml"
        if not meta.exists():
            missing_meta.append(lesson_dir.relative_to(root).as_posix())
    if missing_meta:
        head = ", ".join(missing_meta[:8])
        extra = f", ... ({len(missing_meta) - 8} more)" if len(missing_meta) > 8 else ""
        return f"missing_meta:{head}{extra}", len(lesson_dirs)

    return None, len(lesson_dirs)
