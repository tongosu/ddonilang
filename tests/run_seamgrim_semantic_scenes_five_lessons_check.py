#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_semantic_scenes_five_lessons_v1"
LESSONS = [
    "rep_math_function_line_v1",
    "rep_phys_pendulum_wave1_v1",
    "rep_phys_projectile_xy_v1",
    "rep_phys_damped_oscillation_v1",
    "rep_econ_supply_demand_tax_v1",
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


def check_files_and_contract() -> int:
    required = [
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "curriculum_semantic_scene.js",
        ROOT / "tests" / "seamgrim_semantic_scenes_five_lessons_runner.mjs",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_SEMANTIC_SCENES_MISSING", repr(missing))
    contract = json.loads(read(PACK / "contract.detjson"))
    if contract.get("schema") != "seamgrim.semantic_scenes_five_lessons.contract.v1":
        return fail("E_SEMANTIC_SCENES_CONTRACT_SCHEMA", str(contract.get("schema")))
    if contract.get("product_schema") != "seamgrim.curriculum_semantic_scene.v0":
        return fail("E_SEMANTIC_SCENES_PRODUCT_SCHEMA", str(contract.get("product_schema")))
    if contract.get("lesson_ids") != LESSONS:
        return fail("E_SEMANTIC_SCENES_LESSONS", repr(contract.get("lesson_ids")))
    if int(contract.get("minimum_anchor_count") or 0) < 2:
        return fail("E_SEMANTIC_SCENES_ANCHOR_MIN", repr(contract.get("minimum_anchor_count")))
    if contract.get("runtime_binding_scope") != ["slider_panel", "committed_observation_payload", "primary_view"]:
        return fail("E_SEMANTIC_SCENES_RUNTIME_SCOPE", repr(contract.get("runtime_binding_scope")))
    boundary = contract.get("claim_boundary", {})
    for key in [
        "world_state_storage",
        "state_hash_formula_change",
        "semantic_runtime_completion_claim",
        "new_ddn_syntax",
    ]:
        if boundary.get(key) is not False:
            return fail("E_SEMANTIC_SCENES_BOUNDARY", f"{key}={boundary.get(key)!r}")
    return 0


def check_product_tokens() -> int:
    module_js = read(ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "curriculum_semantic_scene.js")
    run_js = read(ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "run.js")
    app_js = read(ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js")
    snapshot_js = read(ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "project_snapshot.js")
    for token in [
        "CURRICULUM_SEMANTIC_SCENE_SCHEMA",
        "buildCurriculumSemanticSceneV0",
        "summarizeCurriculumSemanticScene",
        "committed_observation_payload",
        "target_selector",
        "runtime_binding",
        "state_hash_participation: false",
    ]:
        if token not in module_js:
            return fail("E_SEMANTIC_SCENES_MODULE", token)
    for token in [
        "buildCurriculumSemanticSceneV0",
        "exportCurriculumSemanticScene",
        "__SEAMGRIM_CURRICULUM_SEMANTIC_SCENE__",
        "data-curriculum-semantic-scene",
        "curriculumSemanticSceneAnchorCount",
        "curriculumSemanticSceneRuntimeBound",
        "publishCurriculumSemanticScene",
    ]:
        if token not in run_js:
            return fail("E_SEMANTIC_SCENES_RUN_JS", token)
    if "exportCurriculumSemanticScene" not in app_js:
        return fail("E_SEMANTIC_SCENES_APP", "save hook missing")
    if "curriculum_semantic_scene" not in snapshot_js:
        return fail("E_SEMANTIC_SCENES_SNAPSHOT", "snapshot field missing")
    return 0


def run_gate(cmd: list[str], code: str, timeout: int) -> int:
    proc = run(cmd, timeout=timeout)
    if proc.returncode != 0:
        return fail(code, f"{' '.join(cmd)}\n{proc.stdout.strip()}")
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_SEMANTIC_SCENES_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_SEMANTIC_SCENES_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    for check in (check_files_and_contract, check_product_tokens):
        rc = check()
        if rc:
            return rc
    gates = [
        (["node", "--check", "solutions/seamgrim_ui_mvp/ui/curriculum_semantic_scene.js"], "E_SEMANTIC_SCENES_MODULE_NODE", 60),
        (["node", "--check", "tests/seamgrim_semantic_scenes_five_lessons_runner.mjs"], "E_SEMANTIC_SCENES_RUNNER_NODE", 60),
        (["node", "tests/seamgrim_semantic_scenes_five_lessons_runner.mjs"], "E_SEMANTIC_SCENES_BROWSER", 420),
        (["python", "tests/run_pack_golden.py", "seamgrim_semantic_scenes_five_lessons_v1"], "E_SEMANTIC_SCENES_GOLDEN", 300),
        (["node", "tests/seamgrim_project_snapshot_runner.mjs"], "E_SEMANTIC_SCENES_PROJECT_SNAPSHOT", 120),
    ]
    for cmd, code, timeout in gates:
        rc = run_gate(cmd, code, timeout)
        if rc:
            return rc
    rc = check_docs_ssot_clean()
    if rc:
        return rc
    print("[seamgrim-semantic-scenes-five-lessons-v1] OK behavior-closed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
