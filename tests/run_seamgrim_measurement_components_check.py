#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_measurement_components_v1"
LESSON = ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons" / "rep_phys_pendulum_wave1_v1"


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
        LESSON / "view_plan.detjson",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "view_plan_runtime.js",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "run.js",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html",
        ROOT / "tests" / "seamgrim_measurement_components_runner.mjs",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_SEAMGRIM_MEASURE_COMPONENTS_MISSING", repr(missing))
    contract = json.loads(read(PACK / "contract.detjson"))
    if contract.get("gate") != "G4" or contract.get("view_plan_schema") != "seamgrim.view_plan.v1":
        return fail("E_SEAMGRIM_MEASURE_COMPONENTS_CONTRACT", repr(contract))
    if contract.get("measurement_components") != ["angle", "length", "vector", "axes", "trajectory"]:
        return fail("E_SEAMGRIM_MEASURE_COMPONENTS_LIST", repr(contract.get("measurement_components")))
    for key in ("state_hash_formula_change", "state_hash_participation", "world_state_stores_view_objects", "parser_change", "runtime_semantics_change", "new_ddn_syntax", "docs_ssot_change"):
        if contract.get(key) is not False:
            return fail("E_SEAMGRIM_MEASURE_COMPONENTS_BOUNDARY", f"{key}={contract.get(key)!r}")
    if contract.get("view_hash_candidate_only") is not True:
        return fail("E_SEAMGRIM_MEASURE_COMPONENTS_HASH_BOUNDARY", repr(contract))
    return 0


def check_product_sources() -> int:
    view_plan = json.loads(read(LESSON / "view_plan.detjson"))
    measures = {str(row.get("kind")): row for row in view_plan.get("measures", []) if isinstance(row, dict)}
    expected = {"angle", "length", "vector", "axes", "trajectory"}
    if set(measures) != expected:
        return fail("E_SEAMGRIM_MEASURE_COMPONENTS_VIEW_PLAN", repr(sorted(measures)))
    if measures["length"].get("unit") != "m" or measures["vector"].get("unit") != "m/s":
        return fail("E_SEAMGRIM_MEASURE_COMPONENTS_UNITS", repr(measures))
    if measures["axes"].get("tick_step") != 0.5 or measures["trajectory"].get("samples") != 24:
        return fail("E_SEAMGRIM_MEASURE_COMPONENTS_DETERMINISM", repr(measures))

    runtime = read(ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "view_plan_runtime.js")
    for token in [
        "buildLengthMeasureShapes",
        "buildVectorMeasureShapes",
        "buildAxesTickShapes",
        "buildTrajectoryTrailShapes",
        "measure_components",
        "state_hash_participation: false",
    ]:
        if token not in runtime:
            return fail("E_SEAMGRIM_MEASURE_COMPONENTS_RUNTIME", token)

    run_js = read(ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "run.js")
    for token in [
        "viewPlanMeasureVisibility",
        "viewPlanMeasureComponents",
        "chk-view-plan-length-measure",
        "chk-view-plan-vector-measure",
        "chk-view-plan-axes-measure",
        "chk-view-plan-trail-measure",
    ]:
        if token not in run_js:
            return fail("E_SEAMGRIM_MEASURE_COMPONENTS_RUN_JS", token)

    html = read(ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html")
    for token in [
        "lesson-length-measure-toggle",
        "lesson-vector-measure-toggle",
        "lesson-axes-measure-toggle",
        "lesson-trail-measure-toggle",
    ]:
        if token not in html:
            return fail("E_SEAMGRIM_MEASURE_COMPONENTS_HTML", token)
    return 0


def run_gate(cmd: list[str], code: str, timeout: int) -> int:
    proc = run(cmd, timeout=timeout)
    if proc.returncode != 0:
        return fail(code, f"{' '.join(cmd)}: {(proc.stdout or '').strip()}")
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_SEAMGRIM_MEASURE_COMPONENTS_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_SEAMGRIM_MEASURE_COMPONENTS_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    for check in (check_files_and_contract, check_product_sources):
        rc = check()
        if rc:
            return rc
    gates = [
        (["node", "--check", "tests/seamgrim_measurement_components_runner.mjs"], "E_SEAMGRIM_MEASURE_COMPONENTS_NODE_CHECK", 60),
        (["node", "tests/seamgrim_measurement_components_runner.mjs"], "E_SEAMGRIM_MEASURE_COMPONENTS_BROWSER", 240),
        (["python", "tests/run_pack_golden.py", "seamgrim_measurement_components_v1"], "E_SEAMGRIM_MEASURE_COMPONENTS_GOLDEN", 300),
    ]
    for cmd, code, timeout in gates:
        rc = run_gate(cmd, code, timeout)
        if rc:
            return rc
    rc = check_docs_ssot_clean()
    if rc:
        return rc
    print("[seamgrim-measurement-components-v1] OK behavior-closed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
