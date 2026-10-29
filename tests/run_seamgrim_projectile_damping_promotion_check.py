#!/usr/bin/env python
from __future__ import annotations

import json
import math
import re
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_projectile_damping_promotion_v1"
LESSONS = [
    "rep_phys_projectile_xy_v1",
    "rep_phys_damped_oscillation_v1",
]


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def run(cmd: list[str], *, timeout: int = 300) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        timeout=timeout,
    )


def check_files() -> int:
    required = [
        PACK / "README.md",
        PACK / "worlds.detjson",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        ROOT / "tests" / "seamgrim_projectile_damping_promotion_runner.mjs",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons" / "rep_phys_damped_oscillation_v1" / "lesson.ddn",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons" / "rep_phys_damped_oscillation_v1" / "meta.toml",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons" / "rep_phys_damped_oscillation_v1" / "view_plan.detjson",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons" / "rep_phys_damped_oscillation_v1" / "text.md",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons" / "rep_phys_damped_oscillation_v1" / "inputs" / "preset_1.ddn",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons" / "rep_phys_damped_oscillation_v1" / "checks" / "CHECKLIST.md",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_PROJECTILE_DAMPING_MISSING", repr(missing))
    return 0


def check_manifest_and_catalog() -> int:
    manifest = json.loads(read(PACK / "worlds.detjson"))
    if manifest.get("schema") != "seamgrim.projectile_damping_promotion.v1":
        return fail("E_PROJECTILE_DAMPING_SCHEMA", str(manifest.get("schema")))
    rows = manifest.get("worlds")
    if not isinstance(rows, list) or [str(row.get("lesson_id")) for row in rows] != LESSONS:
        return fail("E_PROJECTILE_DAMPING_WORLD_ORDER", repr(rows))
    boundary = manifest.get("claim_boundary", {})
    for key in [
        "canonical_curriculum_alignment",
        "external_student_success_claim",
        "state_hash_formula_change",
        "new_ddn_syntax",
        "five_lesson_completion",
    ]:
        if boundary.get(key) is not False:
            return fail("E_PROJECTILE_DAMPING_BOUNDARY", f"{key}={boundary.get(key)!r}")

    index = json.loads(read(ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons" / "index.json"))
    by_id = {str(row.get("id")): row for row in index.get("lessons", []) if isinstance(row, dict)}
    allow = json.loads(read(ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons" / "active_allowlist.detjson"))
    allowed = {str(item) for item in allow.get("lesson_ids", [])}
    cards = json.loads(read(ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons" / "lesson_cards.v0.detjson"))
    card_ids = {str(row.get("lesson_id")) for row in cards.get("cards", []) if isinstance(row, dict)}
    for lesson_id in LESSONS:
        if lesson_id not in by_id:
            return fail("E_PROJECTILE_DAMPING_INDEX", lesson_id)
        if lesson_id not in allowed:
            return fail("E_PROJECTILE_DAMPING_ALLOWLIST", lesson_id)
        if lesson_id not in card_ids:
            return fail("E_PROJECTILE_DAMPING_CARD", lesson_id)
    damping = by_id["rep_phys_damped_oscillation_v1"]
    if damping.get("source") != "representative_wave3a" or damping.get("ssot_pack") != "edu_phys_p011_01_underdamped_xt":
        return fail("E_PROJECTILE_DAMPING_DAMPING_LINEAGE", repr(damping))
    damping_card = next(row for row in cards["cards"] if row.get("lesson_id") == "rep_phys_damped_oscillation_v1")
    if damping_card.get("evidence", {}).get("product_allowlist") is not True:
        return fail("E_PROJECTILE_DAMPING_CARD_ALLOWLIST", repr(damping_card.get("evidence")))
    if "seamgrim_projectile_damping_promotion_v1" not in damping_card.get("lesson_refs", {}).get("pack_refs", []):
        return fail("E_PROJECTILE_DAMPING_CARD_PACK_REF", repr(damping_card.get("lesson_refs")))
    lesson_text = read(ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons" / "rep_phys_damped_oscillation_v1" / "lesson.ddn")
    for token in [
        "마디수: 4001.",
        "총에너지 <- 운동에너지 + 탄성에너지.",
        "에너지손실 <- 에너지손실 + 감쇠계수 * v * v * dt.",
        "총에너지: 총에너지.",
        "에너지손실: 에너지손실.",
    ]:
        if token not in lesson_text:
            return fail("E_PROJECTILE_DAMPING_ENERGY_CONTRACT", token)
    madi_match = re.search(r"설정\s*\{.*?마디수:\s*(\d+)\.", lesson_text, re.S)
    dt_match = re.search(r"dt:.*?범위:\s*([0-9.]+)\.\.([0-9.]+)\.", lesson_text)
    t_max_match = re.search(r"t_max:.*?범위:\s*([0-9.]+)\.\.([0-9.]+)\.", lesson_text)
    if not madi_match or not dt_match or not t_max_match:
        return fail("E_PROJECTILE_DAMPING_TIME_RANGE_PARSE", "madi/dt/t_max")
    configured_madi = int(madi_match.group(1))
    required_madi = math.ceil(float(t_max_match.group(2)) / float(dt_match.group(1))) + 1
    if configured_madi < required_madi:
        return fail("E_PROJECTILE_DAMPING_TIME_RANGE", f"{configured_madi} < {required_madi}")
    if "위상" in damping_card.get("display", {}).get("core_concepts", []):
        return fail("E_PROJECTILE_DAMPING_UNOBSERVABLE_PHASE", repr(damping_card.get("display")))
    for observation in ["총에너지", "에너지손실"]:
        if observation not in damping_card.get("allowed_observations", []):
            return fail("E_PROJECTILE_DAMPING_OBSERVATION", observation)
    return 0


def run_gate(cmd: list[str], code: str, timeout: int) -> int:
    proc = run(cmd, timeout=timeout)
    if proc.returncode != 0:
        return fail(code, f"{' '.join(cmd)}\n{proc.stdout.strip()}")
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_PROJECTILE_DAMPING_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_PROJECTILE_DAMPING_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    for check in (check_files, check_manifest_and_catalog):
        rc = check()
        if rc:
            return rc
    gates = [
        (["node", "--check", "tests/seamgrim_projectile_damping_promotion_runner.mjs"], "E_PROJECTILE_DAMPING_NODE", 60),
        (["node", "tests/seamgrim_projectile_damping_promotion_runner.mjs"], "E_PROJECTILE_DAMPING_BROWSER", 300),
        (["python", "tests/run_pack_golden.py", "seamgrim_projectile_damping_promotion_v1"], "E_PROJECTILE_DAMPING_GOLDEN", 300),
        (["python", "tests/run_seamgrim_curriculum_cards_check.py"], "E_PROJECTILE_DAMPING_CARD_REGRESSION", 180),
    ]
    for cmd, code, timeout in gates:
        rc = run_gate(cmd, code, timeout)
        if rc:
            return rc
    rc = check_docs_ssot_clean()
    if rc:
        return rc
    print("[seamgrim-projectile-damping-promotion-v1] OK behavior-closed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
