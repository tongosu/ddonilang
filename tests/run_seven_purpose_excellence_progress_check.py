#!/usr/bin/env python
from __future__ import annotations

import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
REPORT = ROOT / "docs" / "reports" / "audit" / "SEVEN_PURPOSE_EXCELLENCE_PROGRESS_V1.md"
INDEX = ROOT / "docs" / "reports" / "audit" / "INDEX.md"


PURPOSES = [
    "교육/물리",
    "AI Gym",
    "보개/Manim",
    "게임",
    "경제/수학",
    "교착어권",
    "AI-friendly",
]

REQUIRED_HEADERS = [
    "현재 판정",
    "이번 세션 개발 증빙",
    "남은 gap",
    "\"훌륭\" 전환 조건",
    "다음 pack/test/report",
]

TIERS = ["docs-first", "pack-skeleton", "tooling", "behavior"]


def fail(message: str) -> int:
    print(f"seven_purpose_excellence_progress_check: FAIL: {message}", file=sys.stderr)
    return 1


def main() -> int:
    if not REPORT.exists():
        return fail(f"missing report {REPORT}")
    text = REPORT.read_text(encoding="utf-8")
    for header in REQUIRED_HEADERS:
        if header not in text:
            return fail(f"missing required header {header}")
    for purpose in PURPOSES:
        if purpose not in text:
            return fail(f"missing purpose row {purpose}")
    for tier in TIERS:
        if tier not in text:
            return fail(f"missing evidence tier {tier}")
    if "신규 `닫힘-동작` 주장 없음" not in text:
        return fail("report must explicitly block new behavior claim")
    index_text = INDEX.read_text(encoding="utf-8")
    if "SEVEN_PURPOSE_EXCELLENCE_PROGRESS_V1.md" not in index_text:
        return fail("audit index missing report entry")
    print("seven_purpose_excellence_progress_check: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
