#!/usr/bin/env python
from __future__ import annotations

import re
from pathlib import Path


LEGACY_PATTERNS = [
    re.compile(r"^\s*#\s*이름\s*:", re.IGNORECASE),
    re.compile(r"^\s*#\s*설명\s*:", re.IGNORECASE),
    re.compile(r"^\s*#\s*말씨\s*:", re.IGNORECASE),
    re.compile(r"^\s*#\s*사투리\s*:", re.IGNORECASE),
    re.compile(r"^\s*#\s*그래프\s*:", re.IGNORECASE),
    re.compile(r"^\s*#\s*필수보기\s*:", re.IGNORECASE),
    re.compile(r"^\s*#\s*required_views\s*:", re.IGNORECASE),
    re.compile(r"^\s*#\s*필수보개\s*:", re.IGNORECASE),
    re.compile(r"^\s*#\s*control\s*:", re.IGNORECASE),
    re.compile(r"^\s*#\s*조종\s*:", re.IGNORECASE),
    re.compile(r"^\s*#\s*조절\s*:", re.IGNORECASE),
    re.compile(r"^\s*#\s*layout_preset\s*:", re.IGNORECASE),
    re.compile(r"^\s*#\s*physics_backend\s*:", re.IGNORECASE),
    re.compile(r"^\s*#\s*교과\s*:", re.IGNORECASE),
]


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    targets = [
        root / "solutions" / "seamgrim_ui_mvp" / "seed_lessons_v1",
        root / "solutions" / "seamgrim_ui_mvp" / "lessons",
        root / "solutions" / "seamgrim_ui_mvp" / "lessons_rewrite_v1",
        root / "pack" / "edu",
        root / "pack" / "edu_pilot_phys_econ",
        root / "pack",
        root / "docs" / "ssot" / "pack",
    ]

    violations: list[str] = []
    for base in targets:
        if not base.exists():
            continue
        for path in base.rglob("*.ddn"):
            rel = path.relative_to(root).as_posix()
            lines = path.read_text(encoding="utf-8").splitlines()
            for idx, line in enumerate(lines, start=1):
                if any(pattern.search(line) for pattern in LEGACY_PATTERNS):
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
