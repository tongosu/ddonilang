#!/usr/bin/env python3
"""Build batch generation plan from lesson inventory and seed manifest."""

from __future__ import annotations

import argparse
import json
from pathlib import Path


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--inventory",
        default="build/reports/seamgrim_lesson_inventory.json",
        help="Lesson inventory JSON path",
    )
    parser.add_argument(
        "--seed-manifest",
        default="solutions/seamgrim_ui_mvp/seed_lessons_v1/seed_manifest.detjson",
        help="Seed manifest path",
    )
    parser.add_argument(
        "--out",
        default="build/reports/seamgrim_generation_plan_v1.json",
        help="Output plan path",
    )
    args = parser.parse_args()

    inventory = load_json(Path(args.inventory))
    seed_manifest = load_json(Path(args.seed_manifest))
    lessons = list(inventory.get("lessons", []))
    seeds = list(seed_manifest.get("seeds", []))

    subject_template = {}
    seed_ids = set()
    for seed in seeds:
        subject = str(seed.get("subject", "")).strip()
        template = str(seed.get("template_id", "")).strip()
        seed_id = str(seed.get("seed_id", "")).strip()
        if subject and template:
            subject_template[subject] = template
        if seed_id:
            seed_ids.add(seed_id)

    items = []
    for lesson in lessons:
        lesson_id = str(lesson.get("lesson_id", "")).strip()
        subject = str(lesson.get("subject", "")).strip()
        if not lesson_id or lesson_id in seed_ids:
            continue
        template_id = subject_template.get(subject, "generic_series_v1")
        items.append(
            {
                "lesson_id": lesson_id,
                "subject": subject,
                "goal_summary": lesson.get("goal_summary", ""),
                "controls": lesson.get("control_params", []),
                "template_id": template_id,
                "source_lesson_ddn": lesson.get("lesson_ddn_path", ""),
                "target_lesson_ddn": f"solutions/seamgrim_ui_mvp/lessons_rewrite_v1/{lesson_id}/lesson.ddn",
                "target_text_md": f"solutions/seamgrim_ui_mvp/lessons_rewrite_v1/{lesson_id}/text.md",
            }
        )

    payload = {
        "schema": "seamgrim.curriculum.generation_plan.v1",
        "inventory": str(Path(args.inventory).as_posix()),
        "seed_manifest": str(Path(args.seed_manifest).as_posix()),
        "count": len(items),
        "items": items,
    }
    out_path = Path(args.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(f"[ok] wrote {out_path} ({len(items)} items)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
