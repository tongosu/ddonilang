#!/usr/bin/env python3
# edu_simfirst_lint.py
#
# 목적:
# - `#교과모드: simfirst.v1` 마커가 붙은 교과팩이
#   "시뮬레이션-우선(SimFirst) 계약"을 만족하는지 정적 검사한다.
#
# 설계:
# - 파서/AST 없이도 당장 강제할 수 있게, 단순 문자열 규칙으로 시작한다.
# - 추후 Gate0 parser가 안정되면 AST 기반으로 업그레이드한다.

from __future__ import annotations

import argparse
import os
import re
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable, List, Tuple

SIMFIRST_MARKER = r"#교과모드:\s*simfirst\.v1"

# MUST: state evolves across ticks
TIME_MARKERS = [
    r"\(시작\)할때",
    r"\(매마디\)마다",
    r"\b마디번호\b",
    r"\b델타시간\b",
    r"\b게임시간\b",
]

# MUST: uses view keys
VIEW_MARKERS = [
    r"\b보개_그림판_목록\b",
    r"\b보개_그래프_",
    r"\b그래프_",
]

# MUST: deterministic RNG/time only (heuristic)
RNG_MARKERS = [
    r"\b무작위정수\b",
    r"\b무작위선택\b",
    r"\b무작위\b",
]

WALLCLOCK_BAD = [
    r"\bnow\(",
    r"\btime\(",
    r"\bDate\b",
]

README_REQUIRED_HEADING = r"^##\s+셈그림\s+연출\s*$"


@dataclass
class Violation:
    pack_id: str
    file: str
    reason: str


def iter_packs(root: Path) -> Iterable[Path]:
    # scans `docs/ssot/pack/*` for dirs starting with `edu_`
    if not root.exists():
        return
    for p in sorted(root.iterdir()):
        if p.is_dir() and p.name.startswith("edu_"):
            yield p


def read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8", errors="replace")


def has_any(text: str, patterns: List[str]) -> bool:
    return any(re.search(p, text, flags=re.MULTILINE) for p in patterns)


def lint_pack(pack_dir: Path) -> List[Violation]:
    vid = pack_dir.name
    v: List[Violation] = []

    lesson = pack_dir / "lesson.ddn"
    readme = pack_dir / "README.md"

    if not lesson.exists():
        return v  # ignore non-standard packs for now

    t = read_text(lesson)

    # Only enforce on SimFirst-marked lessons (allows incremental adoption).
    if not re.search(SIMFIRST_MARKER, t, flags=re.MULTILINE):
        return v

    if not has_any(t, TIME_MARKERS):
        v.append(Violation(vid, "lesson.ddn", "SimFirst MUST: time/tick marker missing (시작/매마디/마디번호/델타시간/게임시간)."))

    if not has_any(t, VIEW_MARKERS):
        v.append(Violation(vid, "lesson.ddn", "SimFirst MUST: view marker missing (보개_그림판_목록 or 그래프_*/보개_그래프_*)."))

    # Heuristic checks
    if has_any(t, WALLCLOCK_BAD):
        v.append(Violation(vid, "lesson.ddn", "SimFirst MUST: wall-clock usage suspected (now/time/Date)."))

    # If it uses RNG, it must be from deterministic RNG set (best-effort).
    if re.search(r"\b랜덤\b", t) and not has_any(t, RNG_MARKERS):
        v.append(Violation(vid, "lesson.ddn", "RNG marker '랜덤' found but deterministic RNG calls not found (무작위/무작위정수/무작위선택)."))

    # README required
    if not readme.exists():
        v.append(Violation(vid, "README.md", "SimFirst MUST: README.md missing."))
    else:
        r = read_text(readme)
        if not re.search(README_REQUIRED_HEADING, r, flags=re.MULTILINE):
            v.append(Violation(vid, "README.md", "SimFirst MUST: README missing heading '## 셈그림 연출'."))

    return v


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", type=str, default="docs/ssot/pack", help="Root directory that contains edu_* packs.")
    ap.add_argument("--mode", choices=["warn", "error"], default="error", help="warn: exit 0; error: exit 1 if violations exist.")
    args = ap.parse_args()

    root = Path(args.root)
    all_v: List[Violation] = []
    for pack_dir in iter_packs(root):
        all_v.extend(lint_pack(pack_dir))

    if not all_v:
        print("[simfirst-lint] OK: no violations.")
        return 0

    print(f"[simfirst-lint] violations: {len(all_v)}")
    for vv in all_v:
        print(f"- {vv.pack_id}/{vv.file}: {vv.reason}")

    if args.mode == "warn":
        print("[simfirst-lint] WARN mode: exiting 0.")
        return 0
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
