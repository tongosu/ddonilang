#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
import tomllib
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
LESSONS_ROOT = ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons"
INDEX_PATH = LESSONS_ROOT / "index.json"
STATUS_PATH = LESSONS_ROOT / "schema_status.json"
ALLOWLIST_PATH = LESSONS_ROOT / "active_allowlist.detjson"
TEUL_MANIFEST = ROOT / "tools" / "teul-cli" / "Cargo.toml"

TARGETS = [
    {
        "lesson_id": "rep_ddonirang_vol2_filter_v1",
        "required_views": ["table", "text"],
        "must_contain": ["거르기", "길이", "첫번째", "마지막"],
    },
    {
        "lesson_id": "rep_ddonirang_vol2_map_v1",
        "required_views": ["table", "text"],
        "must_contain": ["바꾸기", "길이", "첫번째", "마지막"],
    },
    {
        "lesson_id": "rep_ddonirang_vol2_pipeline_v1",
        "required_views": ["table", "text"],
        "must_contain": ["거르기", "바꾸기", "길이"],
    },
]


def fail(detail: str) -> int:
    print(f"check=seamgrim_vol2_late_series_lesson detail={detail}")
    return 1


def run_teul_check(path: Path) -> tuple[int, str]:
    proc = subprocess.run(
        [
            "cargo",
            "run",
            "--quiet",
            "--manifest-path",
            str(TEUL_MANIFEST),
            "--",
            "check",
            str(path),
        ],
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    return proc.returncode, (proc.stdout or "") + (proc.stderr or "")


def main() -> int:
    if not INDEX_PATH.exists():
        return fail("missing:index.json")
    if not STATUS_PATH.exists():
        return fail("missing:schema_status.json")
    if not ALLOWLIST_PATH.exists():
        return fail("missing:active_allowlist.detjson")

    index = json.loads(INDEX_PATH.read_text(encoding="utf-8"))
    by_id = {
        str(row.get("id", "")).strip(): row
        for row in index.get("lessons", [])
        if str(row.get("id", "")).strip()
    }
    status = json.loads(STATUS_PATH.read_text(encoding="utf-8"))
    status_by_id = {
        str(row.get("lesson_id", "")).strip(): row
        for row in status.get("lessons", [])
        if str(row.get("lesson_id", "")).strip()
    }
    allowlist = json.loads(ALLOWLIST_PATH.read_text(encoding="utf-8"))
    allow_ids = set(allowlist.get("lesson_ids", []))

    for target in TARGETS:
        lesson_id = target["lesson_id"]
        lesson_dir = LESSONS_ROOT / lesson_id
        lesson_path = lesson_dir / "lesson.ddn"
        preview_path = lesson_dir / "lesson.age3.preview.ddn"
        preset_path = lesson_dir / "inputs" / "preset_1.ddn"
        meta_path = lesson_dir / "meta.toml"
        schema_path = lesson_dir / "ddn.schema.json"
        text_path = lesson_dir / "text.md"

        required_files = [lesson_path, preview_path, preset_path, meta_path, schema_path, text_path]
        missing = [str(path.relative_to(ROOT)).replace("\\", "/") for path in required_files if not path.exists()]
        if missing:
            return fail("missing:" + ",".join(missing))

        lesson_text = lesson_path.read_text(encoding="utf-8")
        preview_text = preview_path.read_text(encoding="utf-8")
        preset_text = preset_path.read_text(encoding="utf-8")
        if lesson_text != preview_text:
            return fail(f"preview_not_synced:{lesson_id}")
        if lesson_text != preset_text:
            return fail(f"preset_not_synced:{lesson_id}")

        for token in target["must_contain"]:
            if token not in lesson_text:
                return fail(f"missing_token:{lesson_id}:{token}")

        meta = tomllib.loads(meta_path.read_text(encoding="utf-8"))
        required_views = meta.get("required_views")
        if required_views != target["required_views"]:
            return fail(f"meta_required_views:{lesson_id}:{required_views}")

        row = by_id.get(lesson_id)
        if not isinstance(row, dict):
            return fail(f"index_missing:{lesson_id}")
        if row.get("required_views") != target["required_views"]:
            return fail(f"index_required_views:{lesson_id}:{row.get('required_views')}")
        if str(row.get("source", "")).strip() != "representative_v1":
            return fail(f"index_source:{lesson_id}:{row.get('source')}")

        if lesson_id not in allow_ids:
            return fail(f"allowlist_missing:{lesson_id}")

        status_row = status_by_id.get(lesson_id)
        if not isinstance(status_row, dict):
            return fail(f"schema_status_missing:{lesson_id}")
        if not status_row.get("has_preview"):
            return fail(f"schema_status_preview:{lesson_id}")
        effective_profile = str(status_row.get("effective_profile") or "").strip()
        if effective_profile in {"legacy", "mixed"}:
            return fail(f"schema_status_profile:{lesson_id}:{effective_profile}")

        for path in (lesson_path, preset_path):
            rc, out = run_teul_check(path)
            if rc != 0:
                return fail(f"teul_check_failed:{lesson_id}:{path.name}:{out.strip() or rc}")

    print(f"seamgrim vol2 late series lesson check ok lessons={len(TARGETS)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
