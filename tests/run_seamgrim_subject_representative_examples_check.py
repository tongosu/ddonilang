#!/usr/bin/env python
from __future__ import annotations

import json
import re
import subprocess
import sys
from pathlib import Path

TARGETS = [
    {
        "pack_id": "edu_seamgrim_rep_econ_supply_demand_tax_v1",
        "lesson_id": "rep_econ_supply_demand_tax_v1",
    },
    {
        "pack_id": "edu_seamgrim_rep_math_function_line_v1",
        "lesson_id": "rep_math_function_line_v1",
    },
    {
        "pack_id": "edu_seamgrim_rep_phys_projectile_xy_v1",
        "lesson_id": "rep_phys_projectile_xy_v1",
    },
    {
        "pack_id": "edu_seamgrim_rep_cs_linear_search_timeline_v1",
        "lesson_id": "rep_cs_linear_search_timeline_v1",
    },
    {
        "pack_id": "edu_seamgrim_rep_science_phase_change_timeline_v1",
        "lesson_id": "rep_science_phase_change_timeline_v1",
    },
]

NON_DDN_EMITS = [
    "guseong-flat-json",
    "alrim-plan-json",
    "exec-policy-map-json",
    "maegim-control-json",
]

LEGACY_HEADER_PATTERNS = [
    re.compile(r"^\s*#\s*이름\s*:", re.IGNORECASE),
    re.compile(r"^\s*#\s*설명\s*:", re.IGNORECASE),
    re.compile(r"^\s*#\s*말씨\s*:", re.IGNORECASE),
    re.compile(r"^\s*#\s*그래프\s*:", re.IGNORECASE),
    re.compile(r"^\s*#\s*조종\s*:", re.IGNORECASE),
    re.compile(r"^\s*#\s*required_views\s*:", re.IGNORECASE),
]


def fail(msg: str) -> int:
    print(f"check=seamgrim_subject_representative_examples detail={msg}")
    return 1


def run_cmd(root: Path, cmd: list[str]) -> tuple[int, str]:
    proc = subprocess.run(
        cmd,
        cwd=root,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    output = (proc.stdout or "") + (proc.stderr or "")
    return proc.returncode, output


def has_fallback_warning(text: str) -> bool:
    return "W_CANON_" in text


def has_legacy_header(text: str) -> int | None:
    for i, line in enumerate(text.splitlines(), start=1):
        if any(p.search(line) for p in LEGACY_HEADER_PATTERNS):
            return i
    return None


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    teul_manifest = str((root / "tools" / "teul-cli" / "Cargo.toml").as_posix())

    rc, out = run_cmd(root, ["cargo", "build", "--quiet", "--manifest-path", teul_manifest])
    if rc != 0:
        return fail(f"teul_build_failed:{out.strip() or rc}")

    index_path = root / "solutions" / "seamgrim_ui_mvp" / "lessons" / "index.json"
    if not index_path.exists():
        return fail("index_missing")
    index_payload = json.loads(index_path.read_text(encoding="utf-8"))
    rows = index_payload.get("lessons")
    if not isinstance(rows, list):
        return fail("index_lessons_not_list")
    by_id = {str(r.get("id")): r for r in rows if isinstance(r, dict) and r.get("id")}

    checks = 0
    emits = 0

    for target in TARGETS:
        pack_id = target["pack_id"]
        lesson_id = target["lesson_id"]

        pack_lesson = root / "pack" / pack_id / "lesson.ddn"
        if not pack_lesson.exists():
            return fail(f"pack_lesson_missing:{pack_id}")

        mirror_lesson = root / "solutions" / "seamgrim_ui_mvp" / "lessons" / lesson_id / "lesson.ddn"
        if not mirror_lesson.exists():
            return fail(f"mirror_lesson_missing:{lesson_id}")

        line = has_legacy_header(pack_lesson.read_text(encoding="utf-8"))
        if line is not None:
            return fail(f"legacy_header_pack:{pack_id}:{line}")

        row = by_id.get(lesson_id)
        if not row:
            return fail(f"index_entry_missing:{lesson_id}")

        ssot_pack = str(row.get("ssot_pack", "")).strip()
        if ssot_pack != pack_id:
            return fail(f"index_ssot_pack_mismatch:{lesson_id}:{ssot_pack}:{pack_id}")

        lesson_arg = str(pack_lesson.as_posix())

        for subcmd in (["check", lesson_arg], ["canon", lesson_arg, "--emit", "ddn"], ["run", lesson_arg, "--madi", "1"]):
            rc, out = run_cmd(
                root,
                ["cargo", "run", "--quiet", "--manifest-path", teul_manifest, "--", *subcmd],
            )
            if rc != 0:
                return fail(f"cmd_fail:{pack_id}:{' '.join(subcmd)}:{out.strip() or rc}")
            if subcmd[0] == "canon" and has_fallback_warning(out):
                return fail(f"ddn_fallback_warning:{pack_id}")
            checks += 1

        for emit in NON_DDN_EMITS:
            rc, out = run_cmd(
                root,
                [
                    "cargo",
                    "run",
                    "--quiet",
                    "--manifest-path",
                    teul_manifest,
                    "--",
                    "canon",
                    lesson_arg,
                    "--emit",
                    emit,
                ],
            )
            if rc != 0:
                return fail(f"emit_fail:{pack_id}:{emit}:{out.strip() or rc}")
            if has_fallback_warning(out):
                return fail(f"emit_fallback_warning:{pack_id}:{emit}")
            emits += 1

    print(
        f"check=seamgrim_subject_representative_examples status=ok lessons={len(TARGETS)} checks={checks} non_ddn_emits={emits}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
