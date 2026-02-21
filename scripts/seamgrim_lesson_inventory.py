#!/usr/bin/env python3
"""Generate an inventory report for Seamgrim lessons.

This report is used as migration input for rewriting lessons into the new DDN style.
It does not transform code; it extracts metadata and risk markers from current files.
"""

from __future__ import annotations

import argparse
import json
import re
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Iterable


ASSIGN_EQUAL_RE = re.compile(r"^[^#\n]*=[^=\n]*\.\s*$")
CONTROL_RE = re.compile(r"#control:\s*(.+)")
CONTROL_PARAM_RE = re.compile(r"([A-Za-z_가-힣][A-Za-z0-9_가-힣]*)\s*:")


@dataclass
class LessonInventory:
    lesson_id: str
    subject: str
    lesson_ddn_path: str
    text_md_path: str | None
    goal_summary: str
    control_params: list[str]
    assign_equal_count: int
    assign_left_count: int
    show_count: int
    boim_block_count: int
    tick_block_count: int


def detect_subject(lesson_id: str) -> str:
    lower = lesson_id.lower()
    economy_tokens = (
        "econ",
        "econom",
        "supply",
        "demand",
        "tax",
        "budget",
        "price",
        "market",
        "store",
        "stock",
        "flow",
        "allowance",
    )
    math_tokens = (
        "math",
        "function",
        "graph",
        "line",
        "fraction",
        "parabola",
        "integral",
        "algebra",
        "geometry",
        "vector",
        "matrix",
        "number",
        "mean",
        "percent",
    )
    physics_tokens = (
        "physics",
        "phys",
        "science",
        "projectile",
        "pendulum",
        "spring",
        "harmonic",
        "motion",
        "orbit",
        "wave",
        "circuit",
        "collision",
        "force",
        "energy",
        "thermal",
        "cooling",
    )
    if any(token in lower for token in economy_tokens):
        return "economy"
    if any(token in lower for token in math_tokens):
        return "math"
    if any(token in lower for token in physics_tokens):
        return "physics"
    return "other"


def read_goal_summary(text_path: Path | None) -> str:
    if text_path is None or not text_path.exists():
        return ""
    for raw in text_path.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if not line:
            continue
        if line.startswith("#"):
            return line.lstrip("#").strip()
        return line
    return ""


def extract_control_params(ddn_text: str) -> list[str]:
    for line in ddn_text.splitlines():
        m = CONTROL_RE.search(line)
        if not m:
            continue
        payload = m.group(1)
        params = CONTROL_PARAM_RE.findall(payload)
        return sorted(set(params))
    return []


def count_assign_equal(ddn_text: str) -> int:
    count = 0
    for line in ddn_text.splitlines():
        if ASSIGN_EQUAL_RE.match(line.strip()):
            count += 1
    return count


def collect_lessons(lessons_root: Path) -> Iterable[LessonInventory]:
    for lesson_dir in sorted(p for p in lessons_root.iterdir() if p.is_dir()):
        ddn_path = lesson_dir / "lesson.ddn"
        if not ddn_path.exists():
            continue
        text_path = lesson_dir / "text.md"
        ddn_text = ddn_path.read_text(encoding="utf-8")
        yield LessonInventory(
            lesson_id=lesson_dir.name,
            subject=detect_subject(lesson_dir.name),
            lesson_ddn_path=str(ddn_path.as_posix()),
            text_md_path=str(text_path.as_posix()) if text_path.exists() else None,
            goal_summary=read_goal_summary(text_path if text_path.exists() else None),
            control_params=extract_control_params(ddn_text),
            assign_equal_count=count_assign_equal(ddn_text),
            assign_left_count=ddn_text.count("<-"),
            show_count=ddn_text.count("보여주기."),
            boim_block_count=ddn_text.count("보임"),
            tick_block_count=ddn_text.count("(매마디)마다"),
        )


def summarize(rows: list[LessonInventory]) -> dict[str, object]:
    by_subject: dict[str, int] = {}
    for row in rows:
        by_subject[row.subject] = by_subject.get(row.subject, 0) + 1
    return {
        "total_lessons": len(rows),
        "subjects": by_subject,
        "legacy_equal_lessons": sum(1 for r in rows if r.assign_equal_count > 0),
        "boim_lessons": sum(1 for r in rows if r.boim_block_count > 0),
        "show_lessons": sum(1 for r in rows if r.show_count > 0),
        "tick_lessons": sum(1 for r in rows if r.tick_block_count > 0),
    }


def main() -> int:
    parser = argparse.ArgumentParser(description="Inventory Seamgrim lessons for migration planning.")
    parser.add_argument(
        "--lessons-root",
        default="solutions/seamgrim_ui_mvp/lessons",
        help="Root directory containing lesson folders.",
    )
    parser.add_argument(
        "--out",
        default="build/reports/seamgrim_lesson_inventory.json",
        help="Output JSON path.",
    )
    args = parser.parse_args()

    lessons_root = Path(args.lessons_root)
    if not lessons_root.exists():
        raise SystemExit(f"lessons root not found: {lessons_root}")

    rows = list(collect_lessons(lessons_root))
    payload = {
        "schema": "seamgrim.lesson.inventory.v1",
        "source_root": str(lessons_root.as_posix()),
        "summary": summarize(rows),
        "lessons": [asdict(r) for r in rows],
    }

    out_path = Path(args.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(f"[ok] wrote {out_path} ({len(rows)} lessons)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
