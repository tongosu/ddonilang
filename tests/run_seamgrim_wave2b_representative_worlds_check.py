#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_wave2b_representative_worlds_v1"


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


def check_files_and_contract() -> int:
    required = [
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons" / "rep_phys_pendulum_wave1_v1" / "lesson.ddn",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons" / "rep_phys_projectile_xy_v1" / "lesson.ddn",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons" / "rep_math_function_line_v1" / "lesson.ddn",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "run.js",
        ROOT / "tests" / "seamgrim_wave2b_representative_worlds_runner.mjs",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_SEAMGRIM_W2B_REP_WORLDS_MISSING", repr(missing))
    contract = json.loads(read(PACK / "contract.detjson"))
    if contract.get("gate") != "G7" or len(contract.get("worlds", [])) != 3:
        return fail("E_SEAMGRIM_W2B_REP_WORLDS_CONTRACT", repr(contract))
    ids = [row.get("lesson_id") for row in contract["worlds"]]
    if ids != ["rep_phys_pendulum_wave1_v1", "rep_phys_projectile_xy_v1", "rep_math_function_line_v1"]:
        return fail("E_SEAMGRIM_W2B_REP_WORLDS_IDS", repr(ids))
    for key in ("state_hash_formula_change", "view_objects_in_world_state", "new_ddn_syntax", "docs_ssot_change"):
        if contract.get(key) is not False:
            return fail("E_SEAMGRIM_W2B_REP_WORLDS_BOUNDARY", f"{key}={contract.get(key)!r}")
    return 0


def check_product_sources() -> int:
    run_js = read(ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "run.js")
    for token in [
        "decorateProjectileMeasurementView",
        "vector_arrow",
        "fit_camera",
        "slope_measure",
        "graph_point",
        "axis_ticks",
        "measurement_components",
        "state_hash_participation: false",
    ]:
        if token not in run_js:
            return fail("E_SEAMGRIM_W2B_REP_WORLDS_RUN_JS", token)
    return 0


def run_gate(cmd: list[str], code: str, timeout: int) -> int:
    proc = run(cmd, timeout=timeout)
    if proc.returncode != 0:
        return fail(code, f"{' '.join(cmd)}: {(proc.stdout or '').strip()}")
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_SEAMGRIM_W2B_REP_WORLDS_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_SEAMGRIM_W2B_REP_WORLDS_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    for check in (check_files_and_contract, check_product_sources):
        rc = check()
        if rc:
            return rc
    gates = [
        (["node", "--check", "solutions/seamgrim_ui_mvp/ui/screens/run.js"], "E_SEAMGRIM_W2B_REP_WORLDS_RUN_NODE", 60),
        (["node", "--check", "tests/seamgrim_wave2b_representative_worlds_runner.mjs"], "E_SEAMGRIM_W2B_REP_WORLDS_RUNNER_NODE", 60),
        (["node", "tests/seamgrim_wave2b_representative_worlds_runner.mjs"], "E_SEAMGRIM_W2B_REP_WORLDS_BROWSER", 360),
        (["python", "tests/run_seamgrim_pendulum_view_plan_product_check.py"], "E_SEAMGRIM_W2B_REP_WORLDS_PENDULUM_REGRESSION", 300),
        (["python", "tests/run_seamgrim_first_delight_projectile_check.py"], "E_SEAMGRIM_W2B_REP_WORLDS_PROJECTILE_REGRESSION", 300),
        (["python", "tests/run_seamgrim_current_line_three_worlds_check.py"], "E_SEAMGRIM_W2B_REP_WORLDS_CURRENT_LINE_REGRESSION", 360),
        (["python", "tests/run_pack_golden.py", "seamgrim_wave2b_representative_worlds_v1"], "E_SEAMGRIM_W2B_REP_WORLDS_GOLDEN", 300),
    ]
    for cmd, code, timeout in gates:
        rc = run_gate(cmd, code, timeout)
        if rc:
            return rc
    rc = check_docs_ssot_clean()
    if rc:
        return rc
    print("[seamgrim-wave2b-representative-worlds-v1] OK behavior-closed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
