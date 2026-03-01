#!/usr/bin/env python
from __future__ import annotations

import re
from pathlib import Path


LEGACY_PATTERNS = [
    re.compile(r"^\s*#\s*control\s*:", re.IGNORECASE),
    re.compile(r"^\s*#\s*조종\s*:", re.IGNORECASE),
    re.compile(r"^\s*#\s*조절\s*:", re.IGNORECASE),
    re.compile(r"^\s*#\s*보개마당_", re.IGNORECASE),
]

LEGACY_SEED_ONLY_PATTERNS = [
    re.compile(r"^\s*#\s*이름\s*:", re.IGNORECASE),
    re.compile(r"^\s*#\s*설명\s*:", re.IGNORECASE),
]

LEGACY_REWRITE_ONLY_PATTERNS = [
    re.compile(r"^\s*#\s*이름\s*:", re.IGNORECASE),
    re.compile(r"^\s*#\s*설명\s*:", re.IGNORECASE),
]

LEGACY_LESSON_PRIMARY_ONLY_PATTERNS = [
    re.compile(r"^\s*#\s*이름\s*:", re.IGNORECASE),
    re.compile(r"^\s*#\s*설명\s*:", re.IGNORECASE),
]


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    targets = [
        root / "solutions" / "seamgrim_ui_mvp" / "seed_lessons_v1",
        root / "solutions" / "seamgrim_ui_mvp" / "lessons",
        root / "solutions" / "seamgrim_ui_mvp" / "lessons_rewrite_v1",
    ]

    violations: list[str] = []
    for base in targets:
        if not base.exists():
            continue
        scope_name = base.name
        extra_patterns = []
        if scope_name == "seed_lessons_v1":
            extra_patterns = LEGACY_SEED_ONLY_PATTERNS
        elif scope_name == "lessons_rewrite_v1":
            extra_patterns = LEGACY_REWRITE_ONLY_PATTERNS
        active_patterns = LEGACY_PATTERNS + extra_patterns
        for path in base.rglob("*.ddn"):
            rel = path.relative_to(root).as_posix()
            file_name = path.name.lower()
            file_extra_patterns = list(extra_patterns)
            if scope_name == "lessons" and file_name == "lesson.ddn":
                file_extra_patterns += LEGACY_LESSON_PRIMARY_ONLY_PATTERNS
            active_patterns = LEGACY_PATTERNS + file_extra_patterns
            lines = path.read_text(encoding="utf-8").splitlines()
            for idx, line in enumerate(lines, start=1):
                if any(pattern.search(line) for pattern in active_patterns):
                    violations.append(f"{rel}:{idx}")

    if violations:
        head = ", ".join(violations[:5])
        extra = ""
        if len(violations) > 5:
            extra = f", ... ({len(violations) - 5} more)"
        print(f"check=legacy_control_meta_found detail={head}{extra}")
        print("seamgrim new grammar check failed: legacy control meta found")
        return 1

    print("seamgrim new grammar check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
