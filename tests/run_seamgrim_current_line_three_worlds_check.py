#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_current_line_three_worlds_v1"
WORLDS = [
    ("rep_math_function_line_v1", "edu_seamgrim_rep_math_function_line_v1", "linear-function", "graph_series"),
    ("rep_phys_projectile_xy_v1", "edu_seamgrim_rep_phys_projectile_xy_v1", "projectile", "trajectory_trail"),
    ("rep_econ_supply_demand_tax_v1", "edu_seamgrim_rep_econ_supply_demand_tax_v1", "market-tax", "graph_series"),
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


def check_files_and_manifest() -> int:
    required = [
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "worlds.detjson",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        ROOT / "tests" / "seamgrim_current_line_three_worlds_runner.mjs",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_SEAMGRIM_THREE_WORLDS_MISSING", repr(missing))
    contract = json.loads(read(PACK / "contract.detjson"))
    if contract.get("world_count") != 3 or contract.get("feasibility_grade") != "ga":
        return fail("E_SEAMGRIM_THREE_WORLDS_CONTRACT", repr(contract))
    if contract.get("corpus_inclusion_claim") is not False:
        return fail("E_SEAMGRIM_THREE_WORLDS_CORPUS_OVERCLAIM", repr(contract.get("corpus_inclusion_claim")))
    manifest = json.loads(read(PACK / "worlds.detjson"))
    rows = manifest.get("worlds")
    if not isinstance(rows, list) or len(rows) != 3:
        return fail("E_SEAMGRIM_THREE_WORLDS_MANIFEST", repr(rows))
    expected_ids = [row[0] for row in WORLDS]
    if [str(row.get("lesson_id", "")) for row in rows] != expected_ids:
        return fail("E_SEAMGRIM_THREE_WORLDS_ORDER", repr(rows))
    for row in rows:
        if row.get("feasibility_grade") != "ga" or row.get("representation_kind") != "explicit_tick":
            return fail("E_SEAMGRIM_THREE_WORLDS_GRADE", repr(row))
    return 0


def check_product_sources() -> int:
    index = json.loads(read(ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons" / "index.json"))
    by_id = {str(row.get("id")): row for row in index.get("lessons", []) if isinstance(row, dict)}
    for lesson_id, pack_id, first_id, view_role in WORLDS:
        lesson_dir = ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons" / lesson_id
        pack_dir = ROOT / "pack" / pack_id
        lesson_source = read(lesson_dir / "lesson.ddn")
        pack_source = read(pack_dir / "lesson.ddn")
        preset_source = read(lesson_dir / "inputs" / "preset_1.ddn")
        if lesson_source != pack_source or lesson_source != preset_source:
            return fail("E_SEAMGRIM_THREE_WORLDS_SOURCE_SYNC", lesson_id)
        meta = read(lesson_dir / "meta.toml")
        for token in [f'first_delight_id = "{first_id}"', f'view_roles = ["{view_role}"]', "first_delight_controls"]:
            if token not in meta:
                return fail("E_SEAMGRIM_THREE_WORLDS_META", f"{lesson_id}: {token}")
        row = by_id.get(lesson_id, {})
        if row.get("ssot_pack") != pack_id or not str(row.get("meta_path", "")).endswith(f"/{lesson_id}/meta.toml"):
            return fail("E_SEAMGRIM_THREE_WORLDS_INDEX", f"{lesson_id}: {row}")

    app = read(ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js")
    run_js = read(ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "run.js")
    css = read(ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css")
    for token in ["FIRST_DELIGHT_LESSON_BY_ALIAS", '"linear-function"', '"market-tax"']:
        if token not in app:
            return fail("E_SEAMGRIM_THREE_WORLDS_APP", token)
    for token in ["graph_series", "mainGraphSeriesSegmentCount", "declaredGraphFallback", "lessonGraphAxes", "observation_series", "prevSpace2d"]:
        if token not in run_js:
            return fail("E_SEAMGRIM_THREE_WORLDS_RUN", token)
    if "#screen-run[data-first-delight]" not in css:
        return fail("E_SEAMGRIM_THREE_WORLDS_CSS", "generic first delight selector missing")
    return 0


def run_gate(cmd: list[str], code: str, timeout: int) -> int:
    proc = run(cmd, timeout=timeout)
    if proc.returncode != 0:
        return fail(code, f"{' '.join(cmd)}: {(proc.stdout or '').strip()}")
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_SEAMGRIM_THREE_WORLDS_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_SEAMGRIM_THREE_WORLDS_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    for check in (check_files_and_manifest, check_product_sources):
        rc = check()
        if rc:
            return rc
    gates = [
        (["node", "--check", "tests/seamgrim_current_line_three_worlds_runner.mjs"], "E_SEAMGRIM_THREE_WORLDS_NODE", 60),
        (["node", "tests/seamgrim_current_line_three_worlds_runner.mjs"], "E_SEAMGRIM_THREE_WORLDS_BROWSER", 180),
        (["node", "tests/seamgrim_first_delight_projectile_runner.mjs"], "E_SEAMGRIM_THREE_WORLDS_PROJECTILE", 150),
        (["python", "tests/run_pack_golden.py", "seamgrim_current_line_three_worlds_v1"], "E_SEAMGRIM_THREE_WORLDS_GOLDEN", 300),
        (["python", "tests/run_seamgrim_subject_representative_examples_check.py"], "E_SEAMGRIM_THREE_WORLDS_CLI", 600),
    ]
    for cmd, code, timeout in gates:
        rc = run_gate(cmd, code, timeout)
        if rc:
            return rc
    rc = check_docs_ssot_clean()
    if rc:
        return rc
    print("[seamgrim-current-line-three-worlds-v1] OK behavior-closed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
