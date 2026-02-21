#!/usr/bin/env python3
"""Pick Golden Seed candidates per subject from lesson inventory."""

from __future__ import annotations

import argparse
import json
from dataclasses import asdict, dataclass
from pathlib import Path


@dataclass
class Candidate:
    subject: str
    lesson_id: str
    score: int
    reasons: list[str]
    lesson_ddn_path: str
    goal_summary: str
    control_params: list[str]


def score_lesson(lesson: dict) -> tuple[int, list[str]]:
    lesson_id = str(lesson.get("lesson_id", ""))
    subject = str(lesson.get("subject", "other"))
    score = 0
    reasons: list[str] = []

    if lesson.get("tick_block_count", 0) > 0:
        score += 3
        reasons.append("시간축 있음(매마디)")
    if lesson.get("assign_equal_count", 0) == 0:
        score += 2
        reasons.append("legacy '=' 없음")
    if len(lesson.get("control_params", [])) >= 2:
        score += 2
        reasons.append("조절 파라미터 충분")
    if lesson.get("goal_summary"):
        score += 1
        reasons.append("목표 요약 존재")

    lower = lesson_id.lower()
    if subject == "physics":
        if "pendulum" in lower or "harmonic" in lower:
            score += 5
            reasons.append("물리 seed 적합(진동/진자)")
    elif subject == "math":
        if any(k in lower for k in ("quadratic", "line", "calculus", "integral")):
            score += 5
            reasons.append("수학 seed 적합(함수/미적분)")
    elif subject == "economy":
        if any(k in lower for k in ("supply", "demand", "market", "macro", "abm", "flow")):
            score += 5
            reasons.append("경제 seed 적합(시장/거시/ABM)")

    return score, reasons


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--inventory",
        default="build/reports/seamgrim_lesson_inventory.json",
        help="Inventory JSON path",
    )
    parser.add_argument(
        "--out",
        default="build/reports/seamgrim_golden_seed_candidates.json",
        help="Output JSON path",
    )
    args = parser.parse_args()

    inv_path = Path(args.inventory)
    if not inv_path.exists():
        raise SystemExit(f"inventory not found: {inv_path}")
    inv = json.loads(inv_path.read_text(encoding="utf-8"))
    lessons = inv.get("lessons", [])

    subjects = ["physics", "math", "economy"]
    picked: list[Candidate] = []

    for subject in subjects:
        pool = [x for x in lessons if x.get("subject") == subject]
        ranked: list[tuple[int, list[str], dict]] = []
        for lesson in pool:
            s, reasons = score_lesson(lesson)
            ranked.append((s, reasons, lesson))
        ranked.sort(key=lambda it: (-it[0], it[2].get("lesson_id", "")))
        if not ranked:
            continue
        top = ranked[0]
        picked.append(
            Candidate(
                subject=subject,
                lesson_id=top[2].get("lesson_id", ""),
                score=top[0],
                reasons=top[1],
                lesson_ddn_path=top[2].get("lesson_ddn_path", ""),
                goal_summary=top[2].get("goal_summary", ""),
                control_params=top[2].get("control_params", []),
            )
        )

    payload = {
        "schema": "seamgrim.seed.candidates.v1",
        "inventory_path": str(inv_path.as_posix()),
        "candidates": [asdict(x) for x in picked],
    }
    out_path = Path(args.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(f"[ok] wrote {out_path} ({len(picked)} candidates)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
