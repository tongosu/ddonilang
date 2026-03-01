#!/usr/bin/env python
from __future__ import annotations

import re
from pathlib import Path


LEGACY_CONTROL_PATTERNS = [
    re.compile(r"^\s*#\s*control\s*:", re.IGNORECASE),
    re.compile(r"^\s*#\s*조종\s*:", re.IGNORECASE),
    re.compile(r"^\s*#\s*조절\s*:", re.IGNORECASE),
]


def fail(detail: str) -> int:
    print(f"check=pendulum_surface_contract detail={detail}")
    return 1


def extract_tick_block_show_vars(text: str) -> list[str]:
    lines = text.replace("\r\n", "\n").replace("\r", "\n").split("\n")
    in_tick = False
    depth = 0
    in_boim = False
    boim_depth = 0
    out: list[str] = []

    for line in lines:
        if not in_tick:
            if "(매마디)마다" in line:
                in_tick = True
                depth = max(1, line.count("{") - line.count("}") or 1)
            continue

        if in_boim:
            m_boim = re.match(r"^\s*([A-Za-z0-9_가-힣]+)\s*:\s*(.+)\.\s*(?://.*)?$", line)
            if m_boim:
                out.append(m_boim.group(1).strip())
            boim_depth += line.count("{")
            boim_depth -= line.count("}")
            if boim_depth <= 0:
                in_boim = False

        m_boim_head = re.match(r"^\s*보임\s*\{\s*(?://.*)?$", line)
        if m_boim_head:
            in_boim = True
            boim_depth = max(1, line.count("{") - line.count("}") or 1)

        m = re.match(r"^\s*([A-Za-z0-9_가-힣]+)\s+보여주기\.\s*$", line)
        if m:
            out.append(m.group(1).strip())

        depth += line.count("{")
        depth -= line.count("}")
        if depth <= 0:
            break
    return out


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    lesson = root / "solutions" / "seamgrim_ui_mvp" / "seed_lessons_v1" / "physics_pendulum_seed_v1" / "lesson.ddn"
    if not lesson.exists():
        return fail(f"lesson_missing:{lesson.as_posix()}")

    text = lesson.read_text(encoding="utf-8")
    lines = text.splitlines()

    if not re.search(r"^\s*채비\s*:\s*\{", text, re.MULTILINE):
        return fail("prep_block_missing")
    if "(시작)할때" not in text:
        return fail("start_block_missing")
    if "(매마디)마다" not in text:
        return fail("tick_block_missing")
    if not re.search(r"^\s*#기본관찰x\s*:", text, re.MULTILINE):
        return fail("default_x_meta_missing")
    if not re.search(r"^\s*#기본관찰\s*:", text, re.MULTILINE):
        return fail("default_y_meta_missing")
    has_shape_block = bool(re.search(r"^\s*(보개|모양)\s*:?\s*\{", text, re.MULTILINE))
    has_shape_markers = bool(re.search(r'"space2d(\.shape)?"', text)) and bool(re.search(r"보여주기\.", text))
    if not has_shape_block and not has_shape_markers:
        return fail("shape_output_contract_missing")

    for idx, line in enumerate(lines, start=1):
        if any(pattern.search(line) for pattern in LEGACY_CONTROL_PATTERNS):
            return fail(f"legacy_control_meta:{idx}")

    show_vars = extract_tick_block_show_vars(text)
    if len(show_vars) < 6:
        return fail(f"tick_show_vars_too_few:{len(show_vars)}")
    required = {"t", "theta", "omega", "energy"}
    if not required.issubset(set(show_vars)):
        missing = ",".join(sorted(required.difference(set(show_vars))))
        return fail(f"tick_show_vars_missing:{missing}")
    for series_id in ("theta", "omega", "energy"):
        if f'"series:{series_id}"' not in text:
            return fail(f"series_marker_missing:{series_id}")

    if "((theta) sin)" not in text:
        return fail("sin_runtime_form_missing")
    if "((theta) cos)" not in text:
        return fail("cos_runtime_form_missing")

    print("seamgrim pendulum surface contract check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
