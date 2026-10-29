#!/usr/bin/env python
from __future__ import annotations

import re
import subprocess
from pathlib import Path


LEGACY_PATTERN_TEXTS = [
    r"^\s*#\s*이름\s*:",
    r"^\s*#\s*설명\s*:",
    r"^\s*#\s*말씨\s*:",
    r"^\s*#\s*사투리\s*:",
    r"^\s*#\s*그래프\s*:",
    r"^\s*#\s*필수보기\s*:",
    r"^\s*#\s*required_views\s*:",
    r"^\s*#\s*필수보개\s*:",
    r"^\s*#\s*control\s*:",
    r"^\s*#\s*조종\s*:",
    r"^\s*#\s*조절\s*:",
    r"^\s*#\s*범위\s*(?:\(|:)",
    r"^\s*#\s*layout_preset\s*:",
    r"^\s*#\s*physics_backend\s*:",
    r"^\s*#\s*교과\s*:",
]
LEGACY_PATTERNS = [re.compile(pattern, re.IGNORECASE) for pattern in LEGACY_PATTERN_TEXTS]


def scan_legacy_with_rg(root: Path, targets: list[Path]) -> tuple[bool, list[str]]:
    cmd = ["rg", "--pcre2", "--line-number", "--color", "never", "--glob", "*.ddn"]
    for pattern in LEGACY_PATTERN_TEXTS:
        cmd.extend(["--regexp", pattern])
    for target in targets:
        try:
            rel = target.relative_to(root).as_posix()
        except ValueError:
            rel = target.as_posix()
        cmd.append(rel)
    proc = subprocess.run(
        cmd,
        cwd=root,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if proc.returncode == 1:
        return True, []
    if proc.returncode != 0:
        return False, [f"rg_failed:{(proc.stderr or proc.stdout or '').strip() or f'rc={proc.returncode}'}"]
    violations: list[str] = []
    for raw in (proc.stdout or "").splitlines():
        line = raw.strip()
        if not line:
            continue
        parts = line.split(":", 2)
        if len(parts) < 2:
            continue
        rel = parts[0].replace("\\", "/")
        line_no = parts[1].strip()
        if rel and line_no:
            violations.append(f"{rel}:{line_no}")
    return True, violations


def scan_legacy_with_python(root: Path, targets: list[Path]) -> list[str]:
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
    return violations


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

    existing_targets = [base for base in targets if base.exists()]
    rg_ok, rg_result = scan_legacy_with_rg(root, existing_targets)
    if rg_ok:
        violations = rg_result
    else:
        detail = ", ".join(rg_result[:1]) if rg_result else "rg_failed"
        print(f"check=legacy_scan_runner_fallback detail={detail}")
        violations = scan_legacy_with_python(root, existing_targets)

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
