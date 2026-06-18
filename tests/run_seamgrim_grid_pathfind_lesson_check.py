#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
LESSON_ID = "roguelike_grid_pathfind_v1"
LESSON = ROOT / "solutions" / "seamgrim_ui_mvp" / "seed_lessons_v1" / LESSON_ID / "lesson.ddn"
MANIFEST = ROOT / "solutions" / "seamgrim_ui_mvp" / "seed_lessons_v1" / "seed_manifest.detjson"
FEATURED_CATALOG = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "featured_seed_catalog.js"
APP_JS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js"
SAM_FIXTURE = ROOT / "pack" / "input_key_alias_ko_v1" / "sam" / "key_alias.input.bin"


def fail(message: str) -> int:
    print(f"[seamgrim-grid-pathfind-lesson] fail {message}", file=sys.stderr)
    return 1


def run_lesson() -> tuple[int, list[str], str, str]:
    cmd = [
        "cargo",
        "run",
        "-q",
        "--manifest-path",
        "tools/teul-cli/Cargo.toml",
        "--",
        "run",
        str(LESSON.relative_to(ROOT)),
        "--sam",
        str(SAM_FIXTURE.relative_to(ROOT)),
        "--madi",
        "1",
    ]
    proc = subprocess.run(cmd, cwd=ROOT, text=True, encoding="utf-8", errors="replace", capture_output=True)
    lines = [line.strip() for line in proc.stdout.splitlines() if line.strip()]
    return proc.returncode, lines, proc.stdout, proc.stderr


def main() -> int:
    manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
    seed_ids = {str(row.get("seed_id", "")) for row in manifest.get("seeds", []) if isinstance(row, dict)}
    featured = {str(item) for item in manifest.get("featured_seed_ids", [])}
    if LESSON_ID not in seed_ids:
        return fail("manifest_seed_missing")
    if LESSON_ID not in featured:
        return fail("featured_seed_missing")
    if not LESSON.exists():
        return fail("lesson_missing")
    if not FEATURED_CATALOG.exists():
        return fail("featured_catalog_missing")
    featured_catalog_text = FEATURED_CATALOG.read_text(encoding="utf-8")
    if f'"{LESSON_ID}"' not in featured_catalog_text:
        return fail("featured_catalog_id_missing")
    app_js_text = APP_JS.read_text(encoding="utf-8")
    if "await mergeSeedLessonsIntoCatalog(merged" not in app_js_text:
        return fail("seed_merge_missing")
    if "if (featuredOnly && !FEATURED_SEED_IDS.includes(id)) return;" not in app_js_text:
        return fail("featured_seed_filter_missing")
    if not SAM_FIXTURE.exists():
        return fail("sam_fixture_missing")

    rc, lines, stdout, stderr = run_lesson()
    if rc != 0:
        print(stdout, end="")
        print(stderr, end="", file=sys.stderr)
        return fail(f"cli_run_failed:{rc}")
    second_rc, second_lines, second_stdout, second_stderr = run_lesson()
    if second_rc != 0:
        print(second_stdout, end="")
        print(second_stderr, end="", file=sys.stderr)
        return fail(f"cli_replay_failed:{second_rc}")
    if lines != second_lines:
        return fail("sam_replay_output_mismatch")
    if not any(line == LESSON_ID for line in lines):
        return fail("lesson_id_output_missing")
    if "차림[1, 0]" not in lines:
        return fail("input_direction_missing")
    if not any(line.startswith("차림[차림[1, 0]") and "차림[4, 3]" in line for line in lines):
        return fail("path_output_missing")
    if "1" not in lines or "0" not in lines:
        return fail("player_coordinate_missing")
    if "9" not in lines:
        return fail("path_length_missing")
    grid_lines = [line for line in lines if len(line) == 5 and set(line).issubset(set(".#*PG"))]
    if len(grid_lines) != 4:
        return fail("console_grid_line_count_mismatch")
    joined_grid = "\n".join(grid_lines)
    for marker in ("P", "G", "#", "*"):
        if marker not in joined_grid:
            return fail(f"console_grid_marker_missing:{marker}")
    if ".P#.." not in grid_lines:
        return fail("player_replay_position_missing")
    print("[seamgrim-grid-pathfind-lesson] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
