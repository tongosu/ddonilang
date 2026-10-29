#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_lesson_view_plan_roundtrip_v1"


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
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "lesson_plan_v0.js",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "editor.js",
        ROOT / "tests" / "seamgrim_lesson_view_plan_roundtrip_runner.mjs",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_SEAMGRIM_LESSON_VIEW_ROUNDTRIP_MISSING", repr(missing))
    contract = json.loads(read(PACK / "contract.detjson"))
    if contract.get("gate") != "G6" or contract.get("lesson_plan_schema") != "seamgrim.lesson_plan.v0":
        return fail("E_SEAMGRIM_LESSON_VIEW_ROUNDTRIP_CONTRACT", repr(contract))
    expected = ["scene_ref", "primary_view", "camera_policy", "measurement_components", "reaction_rule_refs", "view_authoring_hash", "view_plan_ref"]
    if contract.get("activity_fields_preserved") != expected:
        return fail("E_SEAMGRIM_LESSON_VIEW_ROUNDTRIP_FIELDS", repr(contract.get("activity_fields_preserved")))
    for key in ("test_json_loader_hardcode", "state_hash_formula_change", "parser_ddn_change", "new_ddn_syntax", "docs_ssot_change"):
        if contract.get(key) is not False:
            return fail("E_SEAMGRIM_LESSON_VIEW_ROUNDTRIP_BOUNDARY", f"{key}={contract.get(key)!r}")
    return 0


def check_product_sources() -> int:
    lesson_plan = read(ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "lesson_plan_v0.js")
    for token in [
        "measurement_components",
        "reaction_rule_refs",
        "scene_ref",
        "primary_view",
        "camera_policy",
        "view_authoring_hash",
        "view_plan_ref",
    ]:
        if token not in lesson_plan:
            return fail("E_SEAMGRIM_LESSON_VIEW_ROUNDTRIP_LESSON_PLAN", token)
    editor = read(ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "editor.js")
    for token in [
        "applyViewReactionAuthoringToLessonPlanActivity",
        "lesson-plan-scene-ref",
        "lesson-plan-primary-view",
        "lesson-plan-measurements-field",
        "lesson-plan-reaction-rules-field",
        "view_plan.v1",
    ]:
        if token not in editor:
            return fail("E_SEAMGRIM_LESSON_VIEW_ROUNDTRIP_EDITOR", token)
    return 0


def run_gate(cmd: list[str], code: str, timeout: int) -> int:
    proc = run(cmd, timeout=timeout)
    if proc.returncode != 0:
        return fail(code, f"{' '.join(cmd)}: {(proc.stdout or '').strip()}")
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_SEAMGRIM_LESSON_VIEW_ROUNDTRIP_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_SEAMGRIM_LESSON_VIEW_ROUNDTRIP_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    for check in (check_files_and_contract, check_product_sources):
        rc = check()
        if rc:
            return rc
    gates = [
        (["node", "--check", "solutions/seamgrim_ui_mvp/ui/lesson_plan_v0.js"], "E_SEAMGRIM_LESSON_VIEW_ROUNDTRIP_LESSON_PLAN_NODE", 60),
        (["node", "--check", "solutions/seamgrim_ui_mvp/ui/screens/editor.js"], "E_SEAMGRIM_LESSON_VIEW_ROUNDTRIP_EDITOR_NODE", 60),
        (["node", "--check", "tests/seamgrim_lesson_view_plan_roundtrip_runner.mjs"], "E_SEAMGRIM_LESSON_VIEW_ROUNDTRIP_RUNNER_NODE", 60),
        (["node", "tests/seamgrim_lesson_view_plan_roundtrip_runner.mjs"], "E_SEAMGRIM_LESSON_VIEW_ROUNDTRIP_BROWSER", 240),
        (["python", "tests/run_lesson_plan_roundtrip_v1_check.py"], "E_SEAMGRIM_LESSON_VIEW_ROUNDTRIP_BASE_REGRESSION", 300),
        (["python", "tests/run_pack_golden.py", "seamgrim_lesson_view_plan_roundtrip_v1"], "E_SEAMGRIM_LESSON_VIEW_ROUNDTRIP_GOLDEN", 300),
    ]
    for cmd, code, timeout in gates:
        rc = run_gate(cmd, code, timeout)
        if rc:
            return rc
    rc = check_docs_ssot_clean()
    if rc:
        return rc
    print("[seamgrim-lesson-view-plan-roundtrip-v1] OK behavior-closed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
