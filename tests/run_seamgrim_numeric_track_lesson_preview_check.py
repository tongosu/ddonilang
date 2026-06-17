#!/usr/bin/env python3
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "SEAMGRIM_NUMERIC_TRACK_LESSON_PREVIEW_V1.md"
PREV = ROOT / "SEAMGRIM_NUMERIC_TRACK_BROWSER_INDEX_V1.md"
REPORT = ROOT / "docs" / "studio" / "NUMERIC_TRACK_LESSON_PREVIEW_V1.md"
INDEX = ROOT / "docs" / "studio" / "INDEX.md"
PACK = ROOT / "pack" / "seamgrim_numeric_track_lesson_preview_v1"
RUNNER = ROOT / "tests" / "seamgrim_numeric_track_lesson_preview_runner.mjs"
NEXT = "SEAMGRIM_NUMERIC_TRACK_REPORT_EXPORT_V1"


def fail(message: str) -> None:
    print(f"seamgrim_numeric_track_lesson_preview_check: FAIL: {message}", file=sys.stderr)
    sys.exit(1)


def read(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except FileNotFoundError:
        fail(f"missing required file: {path.relative_to(ROOT)}")


def require(path: Path) -> None:
    if not path.exists():
        fail(f"missing required path: {path.relative_to(ROOT)}")


def require_contains(path: Path, tokens: list[str]) -> None:
    text = read(path)
    missing = [token for token in tokens if token not in text]
    if missing:
        fail(f"{path.relative_to(ROOT)} missing tokens: {missing}")


def run(cmd: list[str], *, timeout: int = 180) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        timeout=timeout,
        check=False,
    )


def require_docs_ssot_clean() -> None:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        fail(f"git status docs/ssot failed: {proc.stdout.strip()}")
    if proc.stdout.strip():
        fail(f"docs/ssot changed:\n{proc.stdout}")


def check_required_files() -> None:
    required = [
        DOC,
        PREV,
        REPORT,
        INDEX,
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        RUNNER,
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "numeric_curriculum_track.js",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "browse.js",
        ROOT / "tests" / "run_seamgrim_numeric_track_browser_index_check.py",
    ]
    for path in required:
        require(path)


def check_docs() -> None:
    tokens = [
        "SEAMGRIM_NUMERIC_TRACK_LESSON_PREVIEW_V1",
        "seamgrim.numeric_track_lesson_preview.v1",
        "rep_math_function_line_v1",
        "numeric_root_finding_bisection_v1",
        "linear_inequality_solve_minimum_v1",
        "__SEAMGRIM_NUMERIC_TRACK_DETAIL_PREVIEW__",
        NEXT,
        "docs/ssot/**",
    ]
    require_contains(DOC, tokens + ["No new lesson schema", "No stdlib/parser/runtime change"])
    require_contains(REPORT, tokens + ["No active allowlist mutation", "No automatic solve"])
    require_contains(INDEX, ["SEAMGRIM_NUMERIC_TRACK_LESSON_PREVIEW_V1", "docs/studio/NUMERIC_TRACK_LESSON_PREVIEW_V1.md", "pack/seamgrim_numeric_track_lesson_preview_v1", "tests/run_seamgrim_numeric_track_lesson_preview_check.py"])
    require_contains(ROOT / "STUDIO_LONG_HORIZON_ROADMAP_V1.md", ["SEAMGRIM_NUMERIC_TRACK_LESSON_PREVIEW_V1", NEXT, "numeric track"])
    require_contains(ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md", ["SEAMGRIM_NUMERIC_TRACK_LESSON_PREVIEW_V1", "seamgrim_numeric_track_lesson_preview_v1", NEXT, "docs/ssot/** 변경 없음"])


def check_product_tokens() -> None:
    require_contains(
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "numeric_curriculum_track.js",
        [
            "buildNumericTrackLessonPreview",
            "formatNumericTrackLessonPreviewText",
            "seamgrim.numeric_track_lesson_preview.v1",
            "NUMERIC_TRACK_MODULE_LABELS",
            "numeric_root_finding_bisection_v1",
            "linear_inequality_solve_minimum_v1",
        ],
    )
    require_contains(
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "browse.js",
        [
            "buildNumericTrackLessonPreview",
            "formatNumericTrackLessonPreviewText",
            "__SEAMGRIM_NUMERIC_TRACK_DETAIL_PREVIEW__",
            "__SEAMGRIM_NUMERIC_TRACK_DETAIL_PREVIEW_TEXT__",
            "그래프·표 수업",
            "수치 근거",
            'setElementDatasetValue(this.detailPanelEl, "numericTrack"',
        ],
    )
    require_contains(
        RUNNER,
        [
            "seamgrim_numeric_track_lesson_preview: ok",
            "rep_math_function_line_v1",
            "seamgrim.numeric_track_lesson_preview.v1",
            "numeric_root_finding_bisection_v1",
        ],
    )


def check_contract() -> None:
    payload = json.loads(read(PACK / "contract.detjson"))
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "seamgrim_numeric_track_lesson_preview_v1",
        "kind": "studio_numeric_track_lesson_preview",
        "runtime_claim": False,
        "product_code_change": True,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "closed_by": "SEAMGRIM_NUMERIC_TRACK_LESSON_PREVIEW_V1",
        "based_on": "SEAMGRIM_NUMERIC_TRACK_BROWSER_INDEX_V1",
        "browser_runner": "tests/seamgrim_numeric_track_lesson_preview_runner.mjs",
        "track_id": "studio_numeric_curriculum_track_v1",
        "preview_schema": "seamgrim.numeric_track_lesson_preview.v1",
        "next_item": NEXT,
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if payload.get(key) != value:
            fail(f"contract {key} expected {value!r}, got {payload.get(key)!r}")
    anchors = payload.get("lesson_anchors")
    if anchors != ["rep_math_function_line_v1", "rep_phys_projectile_xy_v1", "rep_econ_supply_demand_tax_v1"]:
        fail(f"unexpected lesson anchors: {anchors!r}")
    covers = payload.get("covers")
    if not isinstance(covers, list) or "numeric_track_detail_panel_sections" not in covers:
        fail(f"contract covers missing detail panel section: {covers!r}")


def check_golden() -> None:
    payload = json.loads(read(PACK / "golden.jsonl").strip())
    expected = [
        "SEAMGRIM_NUMERIC_TRACK_LESSON_PREVIEW_V1",
        "studio numeric track lesson preview sealed",
        "preview schema: seamgrim.numeric_track_lesson_preview.v1",
        f"next: {NEXT}",
    ]
    if payload.get("stdout") != expected:
        fail(f"unexpected golden stdout: {payload.get('stdout')!r}")


def run_required_gates() -> None:
    commands = [
        ["node", "tests/seamgrim_numeric_track_lesson_preview_runner.mjs"],
        ["python", "tests/run_pack_golden.py", "seamgrim_numeric_track_lesson_preview_v1"],
        ["python", "tests/run_seamgrim_numeric_track_browser_index_check.py"],
    ]
    for cmd in commands:
        proc = run(cmd, timeout=300)
        if proc.returncode != 0:
            fail(f"{' '.join(cmd)} failed:\n{proc.stdout}")


def main() -> None:
    check_required_files()
    check_docs()
    check_product_tokens()
    check_contract()
    check_golden()
    run_required_gates()
    require_docs_ssot_clean()
    print("seamgrim_numeric_track_lesson_preview_check: ok")


if __name__ == "__main__":
    main()
